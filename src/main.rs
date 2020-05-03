use gio::prelude::*;
use gtk::prelude::*;
use std::process::Command;
use std::thread;

lazy_static::lazy_static! {
    static ref DARK_GRAY: &'static gdk::RGBA = &gdk::RGBA{red: 0.38, green: 0.38, blue: 0.38, alpha: 1.0};
    static ref GRAY: &'static gdk::RGBA = &gdk::RGBA{red: 0.81, green: 0.81, blue: 0.81, alpha: 1.0};
    static ref TRANS_DARK: &'static gdk::RGBA = &gdk::RGBA{red: 0.0, green: 0.0, blue: 0.0, alpha: 0.8};
    static ref TRANS: &'static gdk::RGBA = &gdk::RGBA{red: 1.0, green: 1.0, blue: 1.0, alpha: 0.0};
}

#[derive(Debug, Default, serde::Deserialize)]
struct Rect {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
}

#[derive(Debug, Default, serde::Deserialize)]
struct Workspace {
    id: i32,
    name: String,
    rect: Rect,
    layout: String,
    urgent: bool,
    fullscreen_mode: i8,
    output: String,
    focused: bool,
    visible: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
struct WorkspaceEvent {
    change: String,
    old: Workspace,
    current: Workspace,
}

fn get_workspace_info() -> Vec<Workspace> {
    let data = Command::new("swaymsg")
        .arg("-t")
        .arg("get_workspaces")
        .output();
    data.map(|d| serde_json::from_slice::<Vec<Workspace>>(&d.stdout).unwrap_or(Vec::new()))
        .unwrap_or(Vec::new())
}

fn now() -> String {
    return format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
}

fn listen_workspace_change() -> glib::Receiver<()> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || loop {
        let data = Command::new("swaymsg")
            .arg("-t")
            .arg("subscribe")
            .arg("[\"workspace\"]")
            .output();
        if data.is_ok() {
            tx.send(()).unwrap();
        }
    });

    return rx;
}

fn create_workspace_labels(b: &gtk::Box) {
    b.foreach(|w| {
        b.remove(w);
    });

    for workspace in get_workspace_info() {
        let label = gtk::Label::new(None);
        label.set_text(&workspace.name);
        let color = if workspace.focused { *GRAY } else { *DARK_GRAY };
        label.override_color(gtk::StateFlags::empty(), Some(color));
        label.override_background_color(gtk::StateFlags::empty(), Some(*TRANS));
        b.add(&label);
    }
    b.show_all();
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(app);

    window.set_title("Footbar");
    window.set_position(gtk::WindowPosition::Mouse);
    window.override_background_color(gtk::StateFlags::empty(), Some(*TRANS_DARK));
    // window.fullscreen();
    window.set_default_size(500, 80);
    window.set_modal(true);
    window.set_keep_above(true);
    window.set_gravity(gdk::Gravity::North);
    window.move_(0, 0);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    let ws_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    create_workspace_labels(&ws_box);

    let time = now();
    let dt = gtk::Label::new(None);
    dt.override_background_color(gtk::StateFlags::empty(), Some(*TRANS));
    dt.override_color(gtk::StateFlags::empty(), Some(*GRAY));
    dt.set_text(&time);
    dt.set_justify(gtk::Justification::Right);

    // let overlay = gtk::Overlay::new();

    // window.add(&overlay);

    hbox.add(&ws_box);
    hbox.add(&dt);

    hbox.set_child_pack_type(&dt, gtk::PackType::End);
    hbox.set_child_padding(&dt, 24);
    hbox.set_child_padding(&ws_box, 24);

    window.add(&hbox);
    window.show_all();

    let rx = listen_workspace_change();
    rx.attach(None, move |_| {
        create_workspace_labels(&ws_box);
        glib::Continue(true)
    });

    gtk::timeout_add_seconds(1, move || {
        let time = now();
        dt.set_text(&time);
        glib::Continue(true)
    });
}

fn main() {
    let app = gtk::Application::new(Some("tech.maralla.footbar"), Default::default()).unwrap();

    app.connect_activate(|a| build_ui(a));
    app.run(Default::default());
}
