use gtk::prelude::*;

use harvardciter2::*;

use lazy_static::lazy_static;
use std::sync::Mutex;


const GLADE_SRC: &str = include_str!("harvardciter.glade");

lazy_static! {
    static ref CURR_PROJ_NAME: Mutex<String> = Mutex::new(String::from("error"));
}


fn setup_title_screen(b: &gtk::Builder) {
    let window: gtk::Window = b.get_object("title_window").unwrap();
    let list_box: gtk::ListBox = b.get_object("projects_list").unwrap();
    let create_new_button: gtk::Button = b.get_object("create_new_project").unwrap();

    // Todo: load projects into list
    let test = gtk::Button::with_label("Hi!");
    list_box.prepend(&test);

    set_window_close(&window);

    let b = b.clone();
    create_new_button.connect_clicked(move |_| {
        show_screen(&b, "create_new_window");

        window.hide();
    });
}

fn setup_new_screen(b: &gtk::Builder) {
    let window: gtk::Window = b.get_object("create_new_window").unwrap();
    let confirm_create_project: gtk::Button = b.get_object("confirm_create_project").unwrap();
    let name_input: gtk::Entry = b.get_object("project_name_input").unwrap();
    let back: gtk::Button = b.get_object("create_project_back").unwrap();

    set_window_close(&window);

    let b2 = b.clone();
    let window2 = window.clone();
    back.connect_clicked(move |_| {
        show_screen(&b2, "title_window");

        window2.hide();
    });

    let proj_name: gtk::Label = b.get_object("project_name").unwrap();
    let proj_window: gtk::Window = b.get_object("project_window").unwrap();

    let b2 = b.clone();
    let window2 = window.clone();
    confirm_create_project.connect_clicked(move |_| {
        let new_proj_name = name_input.get_text();
        let new_proj_name = new_proj_name.as_str();

        proj_name.set_text(new_proj_name);
        proj_window.set_title(format!("{} - Harvard Citer", new_proj_name).as_str());

        let mut curr_proj_name = CURR_PROJ_NAME.lock().unwrap();
        *curr_proj_name = String::from(new_proj_name);

        show_screen(&b2, "project_window");

        window2.hide();
    });
}

fn setup_project_screen(b: &gtk::Builder) {
    let window: gtk::Window = b.get_object("project_window").unwrap();
    let create_reference: gtk::Button = b.get_object("create_reference").unwrap();
    let back: gtk::Button = b.get_object("project_back").unwrap();

    set_window_close(&window);

    let b2 = b.clone();
    let window2 = window.clone();
    back.connect_clicked(move |_| {
        show_screen(&b2, "title_window");

        window2.hide();
    });

    let b2 = b.clone();
    create_reference.connect_clicked(move |_| {
        show_screen(&b2, "reference_window");
    });
}


fn setup_reference_screen(b: &gtk::Builder) {
    let window: gtk::Window = b.get_object("reference_window").unwrap();
    let url: gtk::Entry = b.get_object("url_entry").unwrap();
    let author: gtk::Entry = b.get_object("author_entry").unwrap();
    let year: gtk::Entry = b.get_object("year_entry").unwrap();
    let page: gtk::Entry = b.get_object("page_entry").unwrap();
    let site: gtk::Entry = b.get_object("site_entry").unwrap();
    let accessed: gtk::Entry = b.get_object("accessed_entry").unwrap();

    let guess_button: gtk::Button = b.get_object("guess_fields").unwrap();
    let set_today_button: gtk::Button = b.get_object("set_today").unwrap();

    accessed.set_text(curr_time().as_str());

    set_today_button.connect_clicked(move |_| {
        accessed.set_text(curr_time().as_str());
    });

    let fail: gtk::MessageDialog = b.get_object("fail_guess_dialog").unwrap();
    let fail_close_button: gtk::Button = b.get_object("close_fail_guess").unwrap();

    let fail2 = fail.clone();

    guess_button.connect_clicked(move |_| {
        let results = guess_from_url(&url.get_text());

        match results {
            Ok(guesses) => {
                author.set_text(&guesses.author);
                year.set_text(&guesses.year);
                page.set_text(&guesses.page);
                site.set_text(&guesses.site);
            }
            Err(_) => {
                // Todo; better error messages
                fail2.show_all();
            }
        };
    });

    fail_close_button.connect_clicked(move |_| {
        fail.hide();
    });
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }


    let builder = gtk::Builder::from_string(GLADE_SRC);

    setup_title_screen(&builder);
    setup_new_screen(&builder);
    setup_project_screen(&builder);
    setup_reference_screen(&builder);

    show_screen(&builder, "title_window");

    gtk::main();
}
