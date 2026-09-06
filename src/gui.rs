use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Duration;

use eframe::egui;
use windows::Win32::Foundation::HWND;

/// The single eframe App. The *root* viewport (configured in main.rs's NativeOptions) is a 1x1,
/// off-screen, undecorated window that is deliberately kept genuinely visible (never hidden) -- it never
/// shows anything itself. Both real UI surfaces, the management window and the cycle overlay, are child
/// immediate viewports created only on the frames we want them to exist.
///
/// This indirection isn't cosmetic: painting an *invisible* window on Windows goes through a different
/// internal eframe code path (`check_redraw_requests`'s direct-paint branch, a workaround for a separate
/// Windows-specific bug: https://github.com/emilk/egui/issues/5229) that does not set up the event-loop
/// context `Context::show_viewport_immediate` needs, and calling it from there panics with "egui backend
/// is implemented incorrectly - the user callback was never called". A root that's always genuinely
/// visible (just imperceptible) never takes that branch, so its child-viewport calls always go through
/// the normal, correctly-wrapped window-event path.
#[derive(Default)]
pub struct JellyfishApp {
    icon_cache: HashMap<isize, egui::TextureHandle>,
}

impl JellyfishApp {
    /// Lazily loads and caches a small icon texture for `handle`. Returns an owned (cheap, Arc-backed)
    /// clone so callers never need to hold a borrow of `self` across other work.
    fn icon_texture(&mut self, ctx: &egui::Context, handle: isize) -> Option<egui::TextureHandle> {
        if !self.icon_cache.contains_key(&handle) {
            let (w, h, rgba) = crate::icon::get_window_icon_rgba(HWND(handle))?;
            let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            let tex = ctx.load_texture(format!("wicon-{handle}"), image, egui::TextureOptions::default());
            self.icon_cache.insert(handle, tex);
        }
        self.icon_cache.get(&handle).cloned()
    }
}

impl eframe::App for JellyfishApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Some(mgr) = crate::GLOBAL_MANAGER.get() else {
            return;
        };
        // Clone the manager out (cheap -- its fields are individually Arc-wrapped) so the rest of this
        // frame's work doesn't hold the outer GLOBAL_MANAGER lock and block the input thread.
        let manager = mgr.lock().unwrap().clone();
        let snapshot = manager.snapshot_ordered();
        let current = manager.get_current();

        // Drop icon textures for windows that are no longer tracked.
        self.icon_cache.retain(|h, _| snapshot.iter().any(|(sh, _)| sh == h));

        if crate::MGMT_VISIBLE.load(Ordering::SeqCst) {
            let ctx = ui.ctx().clone();
            let manager = manager.clone();
            let rows: Vec<(isize, String, Option<egui::TextureHandle>)> = snapshot
                .iter()
                .map(|(handle, win)| (*handle, win.title.clone(), self.icon_texture(&ctx, *handle)))
                .collect();

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("jellyfishnet_management"),
                egui::ViewportBuilder::default()
                    .with_title("JellyfishNet")
                    .with_inner_size([420.0, 520.0])
                    .with_active(true),
                move |ctx, _class| {
                    // Closing or minimizing just stops us from calling show_viewport_immediate on
                    // subsequent frames (below), which is how an immediate viewport "disappears" -- no
                    // taskbar entry, no lingering minimized window, and the process keeps running.
                    let close_requested = ctx.input(|i| {
                        i.viewport()
                            .events
                            .iter()
                            .any(|e| matches!(e, egui::ViewportEvent::Close))
                    });
                    let minimized = ctx.input(|i| i.viewport().minimized) == Some(true);
                    if close_requested {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    }
                    if close_requested || minimized {
                        crate::MGMT_VISIBLE.store(false, Ordering::SeqCst);
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("JellyfishNet");
                        ui.label("Windows caught with Alt+B, in cycle order:");
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (handle, title, tex) in &rows {
                                ui.horizontal(|ui| {
                                    if let Some(tex) = tex {
                                        ui.add(
                                            egui::Image::from_texture(tex)
                                                .fit_to_exact_size(egui::vec2(20.0, 20.0)),
                                        );
                                    }
                                    let marker = if Some(*handle) == current { ">" } else { " " };
                                    ui.label(format!("{marker} {title}"));
                                    if ui.small_button("Jump").clicked() {
                                        crate::utils::focus_window(HWND(*handle));
                                        manager.set_current(Some(*handle));
                                    }
                                    if ui.small_button("Up").clicked() {
                                        manager.move_up(*handle);
                                    }
                                    if ui.small_button("Down").clicked() {
                                        manager.move_down(*handle);
                                    }
                                    if ui.small_button("Remove").clicked() {
                                        manager.remove(*handle);
                                    }
                                });
                            }
                        });
                    });
                },
            );
        }

        // The hold-to-preview overlay: a child viewport, shown only while a session is active and has
        // lasted long enough to not be a quick instant-cycle tap.
        let session = crate::PREVIEW_SESSION.get().and_then(|s| *s.lock().unwrap());
        if let Some(session) = session {
            let debounce = Duration::from_millis(150);
            let elapsed = session.started_at.elapsed();
            if elapsed < debounce {
                // Not past the debounce yet: request_repaint() (called when the session started) only
                // asks for an immediate repaint, which is too early to pass this check -- without also
                // scheduling a repaint for when the debounce *will* have elapsed, nothing would ever look
                // again and the overlay would never appear for a hold that's still short right now but
                // ends up going past 150ms.
                ui.ctx().request_repaint_after(debounce - elapsed);
            } else {
                // Pre-resolve icon textures so the overlay closure below doesn't need to borrow `self`.
                let rows: Vec<(String, Option<egui::TextureHandle>, bool)> = snapshot
                    .iter()
                    .map(|(handle, win)| {
                        let tex = self.icon_texture(ui.ctx(), *handle);
                        (win.title.clone(), tex, Some(*handle) == session.preview_handle)
                    })
                    .collect();

                ui.ctx().show_viewport_immediate(
                    egui::ViewportId::from_hash_of("jellyfishnet_overlay"),
                    egui::ViewportBuilder::default()
                        .with_title("JellyfishNet cycle")
                        .with_decorations(false)
                        .with_always_on_top()
                        .with_taskbar(false)
                        .with_resizable(false)
                        .with_inner_size([320.0, 400.0]),
                    move |ctx, _class| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            for (title, tex, highlighted) in &rows {
                                ui.horizontal(|ui| {
                                    if let Some(tex) = tex {
                                        ui.add(
                                            egui::Image::from_texture(tex)
                                                .fit_to_exact_size(egui::vec2(20.0, 20.0)),
                                        );
                                    }
                                    let text = if *highlighted {
                                        egui::RichText::new(title).strong().size(16.0)
                                    } else {
                                        egui::RichText::new(title)
                                    };
                                    ui.label(text);
                                });
                            }
                        });
                    },
                );
            }
        }
    }
}
