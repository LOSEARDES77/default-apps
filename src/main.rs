use adw::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

/// (title, desktop-entry category to pick candidates from, MIME types it sets).
/// With no category, candidates are the apps that declare the first MIME type.
const SPECIALS: &[(&str, Option<&str>, &[&str])] = &[
    (
        "Web browser",
        None,
        &["x-scheme-handler/http", "x-scheme-handler/https", "text/html"],
    ),
    ("File manager", None, &["inode/directory"]),
    // ponytail: there is no standard "default terminal" setting. This stores it under
    // x-scheme-handler/terminal (read it with `xdg-mime query default x-scheme-handler/terminal`);
    // write ~/.config/xdg-terminals.list too if xdg-terminal-exec gets installed.
    ("Terminal", Some("TerminalEmulator"), &["x-scheme-handler/terminal"]),
    (
        "Code editor",
        Some("TextEditor"),
        &[
            "text/x-csrc",
            "text/x-chdr",
            "text/x-c++src",
            "text/x-c++hdr",
            "text/rust",
            "text/x-python",
            "text/x-java",
            "text/x-go",
            "text/javascript",
            "text/x-shellscript",
            "application/json",
            "text/css",
            // Markup and documents
            "text/markdown",
            "text/x-rst",
            "text/org",
            "text/x-tex",
            "text/vnd.typst",
            "application/xml",
            "application/xml-dtd",
            "application/xslt+xml",
            // Config and data
            "application/toml",
            "application/yaml",
            "application/json5",
            "application/x-desktop",
            "text/x-systemd-unit",
            "text/x-nix",
            "text/x-dockerfile",
            "text/x-makefile",
            "text/x-cmake",
            "text/x-meson",
            "text/x-patch",
            "application/sql",
        ],
    ),
    ("Text editor", None, &["text/plain"]),
    ("PDF documents", None, &["application/pdf"]),
    ("HTML files", None, &["text/html"]),
    ("HTTP links", None, &["x-scheme-handler/http"]),
    ("HTTPS links", None, &["x-scheme-handler/https"]),
    ("Email", None, &["x-scheme-handler/mailto"]),
    (
        "Image viewer",
        None,
        &[
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/webp",
            "image/bmp",
            "image/avif",
            "image/svg+xml",
        ],
    ),
    (
        "Video player",
        None,
        &[
            "video/mp4",
            "video/x-matroska",
            "video/webm",
            "video/quicktime",
            "video/x-msvideo",
        ],
    ),
    (
        "Music player",
        None,
        &[
            "audio/mpeg",
            "audio/flac",
            "audio/ogg",
            "audio/x-wav",
            "audio/mp4",
            "audio/x-opus+ogg",
        ],
    ),
    (
        "Archives",
        None,
        &[
            "application/zip",
            "application/x-tar",
            "application/x-compressed-tar",
            "application/x-xz-compressed-tar",
            "application/x-7z-compressed",
            "application/vnd.rar",
        ],
    ),
];

/// Callbacks that re-read the current default for every row in the window.
type Refreshers = Rc<RefCell<Vec<Box<dyn Fn()>>>>;

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("dev.loseardes77.DefaultApps")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn has_category(app: &gio::AppInfo, category: &str) -> bool {
    let Some(id) = app.id() else { return false };
    let file = glib::KeyFile::new();
    file.load_from_data_dirs(format!("applications/{id}"), glib::KeyFileFlags::NONE)
        .is_ok()
        && file
            .string_list("Desktop Entry", "Categories")
            .is_ok_and(|cats| cats.iter().any(|c| c.as_str() == category))
}

fn candidates(category: Option<&str>, mimes: &[String]) -> Vec<gio::AppInfo> {
    match category {
        Some(category) => gio::AppInfo::all()
            .into_iter()
            .filter(|a| a.should_show() && has_category(a, category))
            .collect(),
        None => gio::AppInfo::all_for_type(&mimes[0]),
    }
}

fn set_app_icon(image: &gtk::Image, app: &gio::AppInfo) {
    match app.icon() {
        Some(icon) => image.set_from_gicon(&icon),
        None => image.set_icon_name(Some("application-x-executable")),
    }
}

fn choose(
    parent: &adw::ActionRow,
    category: Option<&'static str>,
    mimes: &Rc<Vec<String>>,
    refreshers: &Refreshers,
) {
    let dialog = adw::Dialog::builder()
        .title(parent.title())
        .content_width(440)
        .build();
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .valign(gtk::Align::Start)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();
    list.set_placeholder(Some(&gtk::Label::builder()
        .label("No installed application can handle this")
        .margin_top(24)
        .margin_bottom(24)
        .build()));

    let default = gio::AppInfo::default_for_type(&mimes[0], false).and_then(|a| a.id());
    let mut group: Option<gtk::CheckButton> = None;
    for app in candidates(category, mimes) {
        let check = gtk::CheckButton::builder()
            .active(default.is_some() && default == app.id())
            .valign(gtk::Align::Center)
            .build();
        check.set_group(group.as_ref());
        let row = adw::ActionRow::builder()
            .title(app.name())
            .subtitle(app.id().unwrap_or_default())
            .use_markup(false)
            .activatable_widget(&check)
            .build();
        let icon = gtk::Image::builder().pixel_size(32).build();
        set_app_icon(&icon, &app);
        row.add_prefix(&icon);
        row.add_suffix(&check);
        // Connected after `active` is set so opening the dialog never writes anything.
        check.connect_toggled(glib::clone!(
            #[weak]
            dialog,
            #[weak]
            row,
            #[strong]
            mimes,
            #[strong]
            refreshers,
            move |check| {
                if !check.is_active() {
                    return;
                }
                // Same effect as `xdg-mime default <id> <mime>`: writes ~/.config/mimeapps.list
                let result = mimes.iter().try_for_each(|m| app.set_as_default_for_type(m));
                refreshers.borrow().iter().for_each(|refresh| refresh());
                match result {
                    Ok(()) => {
                        dialog.close();
                    }
                    Err(e) => row.set_subtitle(&format!("Failed: {e}")),
                }
            }
        ));
        list.append(&row);
        group.get_or_insert(check);
    }

    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(520)
            .child(&list)
            .build(),
    ));
    dialog.set_child(Some(&view));
    dialog.present(Some(parent));
}

fn make_row(
    title: &str,
    subtitle: &str,
    category: Option<&'static str>,
    mimes: Vec<String>,
    refreshers: &Refreshers,
) -> adw::ActionRow {
    let mimes = Rc::new(mimes);
    let icon = gtk::Image::builder().pixel_size(32).build();
    let current = gtk::Label::builder()
        .css_classes(["dim-label"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .subtitle_lines(1)
        .use_markup(false)
        .activatable(true)
        .build();
    row.add_prefix(&icon);
    row.add_suffix(&current);
    row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));

    let refresh = {
        let mimes = mimes.clone();
        move || match gio::AppInfo::default_for_type(&mimes[0], false) {
            Some(app) => {
                current.set_label(&app.name());
                set_app_icon(&icon, &app);
            }
            None => {
                current.set_label("Not set");
                icon.set_icon_name(Some("dialog-question-symbolic"));
            }
        }
    };
    refresh();
    refreshers.borrow_mut().push(Box::new(refresh));

    row.connect_activated({
        let refreshers = refreshers.clone();
        move |row| choose(row, category, &mimes, &refreshers)
    });
    row
}

fn build_ui(app: &adw::Application) {
    let refreshers = Refreshers::default();
    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search apps and file types…")
        .build();

    let new_list = || {
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        list.set_filter_func({
            let search = search.clone();
            move |row| {
                let row = row.downcast_ref::<adw::ActionRow>().unwrap();
                format!("{} {}", row.title(), row.subtitle().unwrap_or_default())
                    .to_lowercase()
                    .contains(&search.text().to_lowercase())
            }
        });
        search.connect_search_changed({
            let list = list.clone();
            move |_| list.invalidate_filter()
        });
        list
    };

    let special_list = new_list();
    for &(title, category, mimes) in SPECIALS {
        let mimes: Vec<String> = mimes.iter().map(|m| m.to_string()).collect();
        special_list.append(&make_row(title, &mimes.join(", "), category, mimes, &refreshers));
    }

    // Every MIME type that at least one installed app claims to handle.
    let types: BTreeSet<String> = gio::AppInfo::all()
        .iter()
        .flat_map(|a| a.supported_types())
        .map(String::from)
        .collect();
    let all_list = new_list();
    for mime in types {
        let description = gio::content_type_get_description(&mime);
        all_list.append(&make_row(&description, &mime.clone(), None, vec![mime], &refreshers));
    }

    let heading = |text: &str| {
        gtk::Label::builder()
            .label(text)
            .xalign(0.0)
            .css_classes(["heading"])
            .margin_top(12)
            .build()
    };
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&heading("Default applications"));
    content.append(&special_list);
    content.append(&heading("All file types"));
    content.append(&all_list);

    let header = adw::HeaderBar::builder()
        .title_widget(&adw::Clamp::builder().maximum_size(420).child(&search).build())
        .build();
    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&adw::Clamp::builder().maximum_size(720).child(&content).build())
            .build(),
    ));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Default Apps")
        .default_width(760)
        .default_height(820)
        .content(&view)
        .build();
    search.set_key_capture_widget(Some(&window));
    gtk::prelude::GtkWindowExt::set_focus(&window, Some(&search));
    window.present();
}
