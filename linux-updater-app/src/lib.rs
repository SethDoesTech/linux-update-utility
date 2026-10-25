pub mod backend;
mod operations;

use eframe::egui::{self, Color32};
use operations::{Action, Event};
use std::{fs, path::Path, sync::mpsc, thread, time::Duration};
use zeroize::{Zeroize, Zeroizing};

const THEMES: [(&str, &str); 5] = [
    ("light", "Light"),
    ("dark", "Dark"),
    ("green-phosphor", "Green Phosphor"),
    ("amber-crt", "Amber CRT"),
    ("classic-blue", "Classic Blue"),
];

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let proportional = [
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    let monospace = [
        "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    ];
    add_system_font(
        &mut fonts,
        "system-proportional",
        egui::FontFamily::Proportional,
        &proportional,
    );
    add_system_font(
        &mut fonts,
        "system-monospace",
        egui::FontFamily::Monospace,
        &monospace,
    );
    // Use designed italic glyphs rather than shearing the regular face.
    let italic_family = egui::FontFamily::Name("system-italic".into());
    fonts.families.insert(
        italic_family.clone(),
        fonts.families[&egui::FontFamily::Proportional].clone(),
    );
    add_system_font(
        &mut fonts,
        "system-italic",
        italic_family,
        &[
            "/usr/share/fonts/noto/NotoSans-Italic.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Italic.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
            "/usr/share/fonts/TTF/DejaVuSans-Oblique.ttf",
        ],
    );
    ctx.set_fonts(fonts);
}

fn add_system_font(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    family: egui::FontFamily,
    candidates: &[&str],
) {
    let Some(bytes) = candidates.iter().find_map(|path| fs::read(path).ok()) else {
        return;
    };
    fonts
        .font_data
        .insert(name.to_owned(), egui::FontData::from_owned(bytes).into());
    fonts
        .families
        .entry(family)
        .or_default()
        .insert(0, name.to_owned());
}
fn theme_path() -> std::path::PathBuf {
    operations::xdg_path("XDG_CONFIG_HOME", ".config").join("Seth Langer/Linux Update Utility.conf")
}
fn read_theme() -> String {
    fs::read_to_string(theme_path())
        .unwrap_or_default()
        .lines()
        .find_map(|l| {
            l.strip_prefix("theme=")
                .filter(|v| THEMES.iter().any(|(id, _)| id == v))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "light".into())
}
fn save_theme(theme: &str) -> std::io::Result<()> {
    let path = theme_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, format!("[appearance]\ntheme={theme}\n"))
}
fn apply_theme(ctx: &egui::Context, theme: &str) {
    // Keep each QPalette role from the original application distinct. In
    // particular, AlternateBase is intentionally quieter than Button.
    let (window, base, alternate, control, text, disabled, highlight) = match theme {
        "dark" => (
            [45, 45, 45],
            [30, 30, 30],
            [45, 45, 45],
            [60, 60, 60],
            [240, 240, 240],
            [135, 135, 135],
            [42, 130, 218],
        ),
        "green-phosphor" => (
            [5, 18, 8],
            [0, 10, 2],
            [8, 30, 12],
            [8, 30, 12],
            [93, 255, 127],
            [44, 128, 62],
            [35, 170, 75],
        ),
        "amber-crt" => (
            [24, 13, 0],
            [12, 7, 0],
            [42, 25, 4],
            [42, 25, 4],
            [255, 191, 64],
            [145, 99, 25],
            [205, 120, 20],
        ),
        "classic-blue" => (
            [0, 20, 92],
            [0, 8, 55],
            [0, 34, 125],
            [0, 34, 125],
            [225, 232, 255],
            [125, 145, 200],
            [70, 130, 255],
        ),
        _ => (
            [245, 245, 245],
            [255, 255, 255],
            [235, 235, 235],
            [240, 240, 240],
            [20, 20, 20],
            [110, 110, 110],
            [48, 140, 230],
        ),
    };
    let color = |v: [u8; 3]| Color32::from_rgb(v[0], v[1], v[2]);
    let mut visuals = if theme == "light" {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let border = color(control).lerp_to_gamma(color(text), 0.05);
    let hover_border = color(control).lerp_to_gamma(color(text), 0.13);
    visuals.panel_fill = color(window);
    visuals.window_fill = color(window);
    visuals.extreme_bg_color = color(base);
    visuals.faint_bg_color = color(alternate);
    visuals.override_text_color = None;
    visuals.hyperlink_color = color(highlight);
    visuals.selection.bg_fill = color(highlight);
    visuals.selection.stroke = egui::Stroke::new(
        1.0,
        if matches!(theme, "light" | "dark") {
            Color32::WHITE
        } else {
            color(base)
        },
    );
    visuals.window_corner_radius = egui::CornerRadius::same(10);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.window_stroke = egui::Stroke::new(1.0, border);
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = color(control);
        widget.weak_bg_fill = color(control);
        widget.bg_stroke = egui::Stroke::new(1.0, border);
        widget.fg_stroke = egui::Stroke::new(1.0, color(text));
        widget.corner_radius = egui::CornerRadius::same(6);
        widget.expansion = 0.0;
    }
    visuals.widgets.noninteractive.bg_fill = color(base);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.hovered.bg_fill = color(control).lerp_to_gamma(color(highlight), 0.035);
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, hover_border);
    visuals.widgets.active.bg_fill = color(control).lerp_to_gamma(color(highlight), 0.08);
    visuals.widgets.active.weak_bg_fill = visuals.widgets.active.bg_fill;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, hover_border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, color(text));
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, color(text));
    visuals.widgets.open.bg_fill = color(control).lerp_to_gamma(color(highlight), 0.05);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, hover_border);
    ctx.set_visuals(visuals);
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.interact_size = egui::vec2(24.0, 24.0);
        style.animation_time = 0.14;
        for text_style in [egui::TextStyle::Body, egui::TextStyle::Button] {
            style
                .text_styles
                .insert(text_style, egui::FontId::proportional(14.0));
        }
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(18.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));
        style.visuals.widgets.noninteractive.fg_stroke.color = color(text);
        style.visuals.widgets.inactive.fg_stroke.color = color(text);
        style.visuals.widgets.hovered.fg_stroke.color = color(text);
        style.visuals.widgets.active.fg_stroke.color = color(text);
        style.visuals.widgets.open.fg_stroke.color = color(text);
        style.visuals.warn_fg_color = color(disabled);
    });
}

struct Gui {
    setup: bool,
    logo: egui::TextureHandle,
    theme: String,
    busy: bool,
    receiver: Option<mpsc::Receiver<Event>>,
    output: String,
    message: Option<String>,
    message_action: Option<Action>,
    system_info: Option<String>,
    password_action: Option<Action>,
    password: Zeroizing<String>,
    focus_password: bool,
    password_composing: bool,
    confirm_uninstall: bool,
    close_after_message: bool,
}
impl Gui {
    fn new(ctx: &egui::Context, setup: bool) -> Self {
        configure_fonts(ctx);
        let icon = eframe::icon_data::from_png_bytes(include_bytes!("../luu-icon.png"))
            .expect("bundled icon is valid PNG");
        let logo = ctx.load_texture(
            "luu-logo",
            egui::ColorImage::from_rgba_unmultiplied(
                [icon.width as usize, icon.height as usize],
                &icon.rgba,
            ),
            egui::TextureOptions::LINEAR,
        );
        let theme = read_theme();
        apply_theme(ctx, &theme);
        Self {
            setup,
            logo,
            theme,
            busy: false,
            receiver: None,
            output: String::new(),
            message: None,
            message_action: None,
            system_info: None,
            password_action: None,
            password: Zeroizing::new(String::new()),
            focus_password: false,
            password_composing: false,
            confirm_uninstall: false,
            close_after_message: false,
        }
    }
    fn start(&mut self, action: Action) {
        self.busy = true;
        self.output.clear();
        let password = std::mem::replace(&mut self.password, Zeroizing::new(String::new()));
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        thread::spawn(move || operations::perform(action, password, tx));
    }
    fn authorize(&mut self, action: Action) {
        self.password.zeroize();
        self.password_action = Some(action);
        self.focus_password = true;
        self.password_composing = false;
    }
    fn poll(&mut self) {
        let events: Vec<_> = self
            .receiver
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Output(text) => {
                    self.output.push_str(&text);
                    // Bound long-running package-manager output without interrupting the process.
                    if self.output.len() > 2_000_000 {
                        let mut cut = self.output.len() - 1_000_000;
                        while !self.output.is_char_boundary(cut) {
                            cut += 1;
                        }
                        self.output.drain(..cut);
                    }
                }
                Event::PasswordRequired => self.authorize(Action::Update),
                Event::Done(action, result) => {
                    self.busy = false;
                    self.receiver = None;
                    match result {
                        Ok(text) if action == Action::Info => {
                            self.system_info = Some(text);
                        }
                        Ok(text) if !text.is_empty() => {
                            self.close_after_message =
                                matches!(action, Action::Install | Action::Uninstall);
                            self.message_action = Some(action);
                            self.message = Some(text);
                        }
                        Ok(_) => {}
                        Err(error) => {
                            self.message_action = None;
                            self.message = Some(format!("Operation failed: {error}"));
                        }
                    }
                }
            }
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        if let Some(info) = self.system_info.clone() {
            egui::Modal::new(egui::Id::new("system-info"))
                .backdrop_color(Color32::from_black_alpha(20))
                .show(ctx, |ui| {
                    ui.set_width(450.0);
                    ui.heading("System Info");
                    ui.add_space(12.0);
                    egui::Grid::new("system-info-grid")
                        .num_columns(2)
                        .spacing([22.0, 12.0])
                        .show(ui, |ui| {
                            for (label, value) in system_info_rows(&info) {
                                ui.label(format!("{label}:"));
                                ui.add(egui::Label::new(value).wrap());
                                ui.end_row();
                            }
                        });
                    ui.add_space(14.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if glass_button(ui, "Close", true, egui::vec2(90.0, 32.0)).clicked() {
                            self.system_info = None;
                        }
                    });
                });
        } else if let Some(text) = self.message.clone() {
            egui::Modal::new(egui::Id::new("message"))
                .backdrop_color(Color32::from_black_alpha(20))
                .show(ctx, |ui| {
                    let success = match self.message_action {
                        Some(Action::Generate) => Some((
                            "Script Ready",
                            "Your update script was generated successfully and is ready to use.",
                        )),
                        Some(Action::PrepareUpdate | Action::Update) => Some((
                            "Update Complete",
                            "Your system update finished successfully.",
                        )),
                        _ => None,
                    };
                    let close = if let Some((title, detail)) = success {
                        success_dialog(ui, &self.logo, title, detail)
                    } else {
                        ui.set_max_width(520.0);
                        ui.label(text);
                        ui.add_space(8.0);
                        ui.button("Close").clicked()
                    };
                    if close {
                        self.message = None;
                        self.message_action = None;
                        if self.close_after_message {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
        } else if self.confirm_uninstall {
            egui::Modal::new(egui::Id::new("uninstall"))
                .backdrop_color(Color32::from_black_alpha(20))
                .show(ctx, |ui| {
                ui.set_max_width(420.0);
                ui.heading("Uninstall Linux Update Utility?");
                ui.label("This removes the installed application, launcher, and icon. Your user-local application data will be preserved.");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() { self.confirm_uninstall = false; }
                    if ui.button("Uninstall").clicked() { self.confirm_uninstall = false; self.authorize(Action::Uninstall); }
                });
                });
        } else if let Some(action) = self.password_action {
            egui::Modal::new(egui::Id::new("password"))
                .backdrop_color(Color32::from_black_alpha(20))
                .show(ctx, |ui| {
                    ui.heading("Administrator Authorization");
                    ui.label(match action {
                        Action::Install => "Enter your password to install Linux Update Utility:",
                        Action::Uninstall => {
                            "Enter your password to uninstall Linux Update Utility:"
                        }
                        _ => "Enter your password to authorize system updates:",
                    });
                    let password_id = egui::Id::new("administrator_password");
                    if ui.memory(|memory| memory.has_focus(password_id)) {
                        // egui 0.31 drops commit-only IME input after the first
                        // character because its composition cursor is stale.
                        // Start each standalone commit at the current cursor,
                        // while preserving real preedit/composition sequences.
                        ui.input_mut(|input| {
                            let events = std::mem::take(&mut input.events);
                            for event in events {
                                match &event {
                                    egui::Event::Ime(egui::ImeEvent::Preedit(_)) => {
                                        self.password_composing = true;
                                    }
                                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => {
                                        if !self.password_composing {
                                            input
                                                .events
                                                .push(egui::Event::Ime(egui::ImeEvent::Enabled));
                                        }
                                        self.password_composing = false;
                                    }
                                    egui::Event::Ime(egui::ImeEvent::Disabled) => {
                                        self.password_composing = false;
                                    }
                                    _ => {}
                                }
                                input.events.push(event);
                            }
                        });
                    } else {
                        self.password_composing = false;
                    }
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut *self.password)
                            .id(password_id)
                            .password(true)
                            .hint_text("Password"),
                    );
                    if self.focus_password {
                        response.request_focus();
                        self.focus_password = false;
                    }
                    let submit =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if (ui
                            .add_enabled(!self.password.is_empty(), egui::Button::new("Authorize"))
                            .clicked()
                            || submit)
                            && !self.password.is_empty()
                            && self.password_action.is_some()
                        {
                            self.password_action = None;
                            self.start(action);
                        }
                        if ui.button("Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            self.password.zeroize();
                            self.password_action = None;
                        }
                    });
                });
        }
    }
}

fn system_info_rows(info: &str) -> Vec<(&str, &str)> {
    info.lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(label, value)| (label.trim(), value.trim()))
        .collect()
}

fn success_dialog(
    ui: &mut egui::Ui,
    logo: &egui::TextureHandle,
    title: &str,
    detail: &str,
) -> bool {
    ui.set_width(390.0);
    ui.vertical_centered(|ui| {
        ui.add_space(4.0);
        ui.add(egui::Image::new((logo.id(), egui::vec2(58.0, 58.0))));
        ui.add_space(8.0);
        ui.heading(title);
        ui.add_space(4.0);
        ui.label(egui::RichText::new(detail).size(14.0));
        ui.add_space(14.0);
        glass_button(ui, "Done", true, egui::vec2(120.0, 32.0)).clicked()
    })
    .inner
}

fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    themed_button(ui, label, enabled, 50.0)
}

fn themed_button(ui: &mut egui::Ui, label: &str, enabled: bool, height: f32) -> egui::Response {
    glass_button(ui, label, enabled, egui::vec2(ui.available_width(), height))
}

fn glass_button(ui: &mut egui::Ui, label: &str, enabled: bool, size: egui::Vec2) -> egui::Response {
    let label = egui::RichText::new(label).font(egui::FontId::proportional(14.0));

    // min_size alone inherits the parent's left/top text alignment.
    // add_sized creates a centered, justified layout for the button label.
    ui.add_enabled_ui(enabled, |ui| {
        // Reserve a background layer so the gloss stays behind the label.
        let background = ui.painter().add(egui::Shape::Noop);
        let response = ui.add_sized(
            size,
            egui::Button::new(label)
                .fill(Color32::TRANSPARENT)
                .corner_radius(8),
        );
        let visuals = ui.style().interact(&response);
        let base = visuals.weak_bg_fill;
        let pressed = response.is_pointer_button_down_on();
        let top = base.lerp_to_gamma(Color32::WHITE, if pressed { 0.03 } else { 0.12 });
        let bottom = base.lerp_to_gamma(Color32::BLACK, if pressed { 0.10 } else { 0.04 });
        let rect = response.rect.shrink(1.0);
        let mut shapes = vec![egui::Shape::rect_filled(rect, 7, base)];
        // Inset the sheen to leave the rounded frame clear. Two adjoining
        // gradients create the soft glass highlight of classic desktop controls.
        let glass = rect.shrink(5.0);
        let mut mesh = egui::Mesh::default();
        for (fraction, color) in [
            (0.0, top),
            (0.40, base.lerp_to_gamma(Color32::WHITE, 0.05)),
            (0.65, base),
            (1.0, bottom),
        ] {
            let y = glass.top() + glass.height() * fraction;
            mesh.colored_vertex(egui::pos2(glass.left(), y), color);
            mesh.colored_vertex(egui::pos2(glass.right(), y), color);
        }
        for row in 0..3 {
            let index = row * 2;
            mesh.add_triangle(index, index + 1, index + 2);
            mesh.add_triangle(index + 1, index + 3, index + 2);
        }
        shapes.push(egui::Shape::mesh(mesh));
        shapes.push(egui::Shape::line_segment(
            [
                rect.left_top() + egui::vec2(8.0, 1.0),
                rect.right_top() + egui::vec2(-8.0, 1.0),
            ],
            egui::Stroke::new(1.0, top),
        ));
        ui.painter().set(background, egui::Shape::Vec(shapes));
        response
    })
    .inner
}

fn style_settings_menu(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.visuals.window_fill = style
        .visuals
        .panel_fill
        .lerp_to_gamma(Color32::WHITE, 0.025);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(10);
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(7);
        widget.weak_bg_fill = widget.bg_fill.lerp_to_gamma(Color32::WHITE, 0.06);
    }
}

impl Gui {
    fn credit_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
            ui.painter()
                .circle_filled(rect.center(), 9.0, Color32::WHITE);
            ui.painter().circle_stroke(
                rect.center(),
                8.5,
                egui::Stroke::new(1.0, Color32::from_gray(176)),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "i",
                egui::FontId::proportional(13.0),
                Color32::BLACK,
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "License & Notices")
            });
            if response.on_hover_text("License & Notices").clicked() {
                self.message = Some(
                    "Copyright © 2026 Seth Langer\nLicensed under GNU GPLv3\nLinux Update Utility"
                        .into(),
                );
            }

            let mut credit = egui::text::LayoutJob::default();
            for text in [
                "Created by Seth Langer",
                "  |  RELEASE: 1.2.1  |  Now Built On Rust",
            ] {
                credit.append(
                    text,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(
                            12.0,
                            egui::FontFamily::Name("system-italic".into()),
                        ),
                        color: ui.visuals().text_color(),
                        ..Default::default()
                    },
                );
            }
            ui.label(credit);
        });
    }

    fn render(&mut self, ctx: &egui::Context) {
        self.poll();
        if self.busy {
            ctx.request_repaint_after(Duration::from_millis(50));
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.message =
                    Some("Wait for the current operation to finish before closing.".into());
            }
        }
        if !self.setup {
            egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                style_settings_menu(ui);
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("Settings", |ui| {
                        ui.menu_button("Theme", |ui| {
                            for (index, (id, label)) in THEMES.into_iter().enumerate() {
                                if index == 2 {
                                    ui.separator();
                                }
                                let caption = if self.theme == id {
                                    format!("✓ {label}")
                                } else { label.to_owned() };
                                if glass_button(ui, &caption, true, egui::vec2(180.0, 30.0)).clicked() {
                                    self.theme = id.into(); apply_theme(ctx, id);
                                    if let Err(e) = save_theme(id) { self.message = Some(format!("Cannot save theme: {e}")); }
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.separator();
                        if glass_button(ui, "Uninstall Linux Update Utility…", !self.busy, egui::vec2(250.0, 32.0)).clicked() {
                            if Path::new(operations::INSTALLED).exists() { self.confirm_uninstall = true; }
                            else { self.message = Some("No system-wide installation was found. This may be a development or portable copy.".into()); }
                            ui.close_menu();
                        }
                    });
                });
            });
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(if self.setup {
                        egui::Margin {
                            left: 38,
                            right: 38,
                            top: 32,
                            bottom: 24,
                        }
                    } else {
                        egui::Margin { left: 9, right: 9, top: 9, bottom: 24 }
                    }),
            )
            .show(ctx, |ui| {
                if self.setup {
                    let installed = Path::new(operations::INSTALLED).exists();
                    // Lay out the footer from the bottom so wrapped credits
                    // cannot push it past the window's reserved bottom margin.
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    self.credit_row(ui);
                    ui.add_space(12.0);
                    ui.add_enabled_ui(!self.busy, |ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if glass_button(ui,
                                        if installed {
                                            "Close"
                                        } else {
                                            "No, Do Nothing"
                                        }, true, egui::vec2(150.0, 32.0))
                                    .clicked()
                                {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                if glass_button(ui,
                                        if installed {
                                            "Update Existing Install"
                                        } else {
                                            "Yes, Set Up"
                                        }, true, egui::vec2(170.0, 32.0))
                                    .clicked()
                                {
                                    self.authorize(Action::Install);
                                }
                            },
                        );
                    });
                    ui.add_space(12.0);
                    let remaining = ui.available_size();
                    ui.allocate_ui_with_layout(remaining, egui::Layout::top_down(egui::Align::Min), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(if installed { "Linux Update Utility is already installed on this system.\n\nSelect \"Update Existing Install\" to replace it with the version included in this setup utility." } else { "Would you like to set up Linux Update Utility?\n\nIf yes, the app will be available like any other application. To get started, press \"Generate Script\" and then \"Update System\".\n\nIf no, the setup utility will make no changes to your system." });
                        });
                    });
                    });
                } else {
                    ui.add_space(40.0);
                    if primary_button(ui, "Generate Script", !self.busy).clicked() {
                        self.start(Action::Generate);
                    }
                    ui.add_space(15.0);
                    if primary_button(ui, "Update System", !self.busy).clicked() {
                        self.start(Action::PrepareUpdate);
                    }
                    if self.busy {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Working…");
                        });
                    }
                    if !self.output.is_empty() {
                        // Reserve the footer before assigning space to the log.
                        // Updates must fit the user's existing window size.
                        let output_height = (ui.available_height() - 70.0).max(12.0);
                        egui::Frame::new()
                            .fill(ui.visuals().extreme_bg_color)
                            .stroke(ui.visuals().widgets.inactive.bg_stroke)
                            .inner_margin(6)
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(
                                    ui.available_width(),
                                    output_height - 12.0,
                                ));
                                egui::ScrollArea::vertical()
                                    .stick_to_bottom(true)
                                    .max_height(output_height - 12.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&self.output).monospace(),
                                            )
                                            .selectable(true),
                                        );
                                    });
                            });
                    }
                    ui.add_space((ui.available_height() - 52.0).max(0.0));
                    self.credit_row(ui);
                    if themed_button(ui, "System Info", !self.busy, 28.0).clicked() {
                        self.start(Action::Info);
                    }
                }
            });
        self.dialogs(ctx);
    }
}

pub fn run(setup: bool) -> eframe::Result {
    let title = if setup {
        "LUU Setup Utility"
    } else {
        "Linux Update Utility"
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../luu-icon.png"))
                    .expect("bundled icon is valid PNG"),
            )
            .with_app_id(if setup {
                "luu-setup-utility"
            } else {
                "linux-update-utility"
            })
            .with_inner_size(if setup {
                [560.0, 330.0]
            } else {
                [600.0, 400.0]
            })
            .with_min_inner_size([420.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| Ok(Box::new(Gui::new(&cc.egui_ctx, setup)))),
    )
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.render(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_button_labels_are_centered_in_the_painted_buttons() {
        for label in ["Generate Script", "Update System"] {
            for width in [420.0, 600.0, 850.0] {
                let ctx = egui::Context::default();
                let mut button_rect = egui::Rect::NOTHING;
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 400.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            button_rect = primary_button(ui, label, true).rect;
                        });
                    },
                );
                let text_center = output
                    .shapes
                    .iter()
                    .find_map(|shape| {
                        if let egui::Shape::Text(text) = &shape.shape {
                            if text.galley.text() == label {
                                return Some(text.pos + text.galley.size() * 0.5);
                            }
                        }
                        None
                    })
                    .expect("button label must be painted");
                assert!(
                    (text_center - button_rect.center()).length() < 1.0,
                    "{label} must be centered at width {width}"
                );
            }
        }
    }

    #[test]
    fn both_windows_and_authorization_render_in_every_theme() {
        for setup in [false, true] {
            for (theme, _) in THEMES {
                let ctx = egui::Context::default();
                let mut app = Gui::new(&ctx, setup);
                apply_theme(&ctx, theme);
                let _ = ctx.run(Default::default(), |ctx| app.render(ctx));
                app.authorize(if setup {
                    Action::Install
                } else {
                    Action::Update
                });
                let output = ctx.run(Default::default(), |ctx| app.render(ctx));
                assert!(!output.shapes.is_empty());
                assert!(!app.busy);
            }
        }
    }

    #[test]
    fn password_accepts_successive_onscreen_keyboard_commits() {
        let ctx = egui::Context::default();
        let mut app = Gui::new(&ctx, false);
        app.authorize(Action::Update);
        for _ in 0..3 {
            let _ = ctx.run(Default::default(), |ctx| app.render(ctx));
        }
        for character in ["a", "b", "ç"] {
            let _ = ctx.run(
                egui::RawInput {
                    events: vec![egui::Event::Ime(egui::ImeEvent::Commit(character.into()))],
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert_eq!(app.password.as_str(), "abç");
        // Real composition spans frames and replaces the preedit selection.
        for event in [
            egui::ImeEvent::Enabled,
            egui::ImeEvent::Preedit("e".into()),
            egui::ImeEvent::Preedit("é".into()),
            egui::ImeEvent::Commit("é".into()),
            egui::ImeEvent::Disabled,
            egui::ImeEvent::Commit("z".into()),
        ] {
            let _ = ctx.run(
                egui::RawInput {
                    events: vec![egui::Event::Ime(event)],
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert_eq!(app.password.as_str(), "abçéz");
        assert!(!app.busy);
    }

    #[test]
    fn authorization_event_does_not_start_update_and_completion_unlocks_ui() {
        let ctx = egui::Context::default();
        let mut app = Gui::new(&ctx, false);
        let (tx, rx) = mpsc::channel();
        app.receiver = Some(rx);
        app.busy = true;
        app.password.push_str("old password");
        tx.send(Event::PasswordRequired).unwrap();
        tx.send(Event::Done(Action::PrepareUpdate, Ok(String::new())))
            .unwrap();
        app.poll();
        assert!(!app.busy);
        assert!(app.password.is_empty());
        assert!(app.password_action == Some(Action::Update));
        assert!(app.receiver.is_none());
    }

    #[test]
    fn system_information_is_split_into_labeled_rows() {
        let rows = system_info_rows(
            "Distribution: Starch Linux\nArchitecture: x86_64\nPackage Managers: pacman, flatpak",
        );
        assert_eq!(
            rows,
            [
                ("Distribution", "Starch Linux"),
                ("Architecture", "x86_64"),
                ("Package Managers", "pacman, flatpak"),
            ]
        );
    }
}
