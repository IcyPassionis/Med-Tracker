#include <gtk/gtk.h>
#include <stdint.h>
#include <string.h>

typedef void (*TrayEventCallback)(int event, uint64_t generation, void *user_data);

enum {
    EVENT_INITIALIZED = 0,
    EVENT_EMBEDDED = 1,
    EVENT_ACTIVATE = 2,
    EVENT_SHOW = 3,
    EVENT_EXIT = 4,
    EVENT_FAILED = 5,
    EVENT_UNEMBEDDED = 6,
};

enum {
    COMMAND_SET_ENABLED = 1,
    COMMAND_QUERY_STATE = 2,
    COMMAND_SHUTDOWN = 3,
};

typedef struct {
    int command;
    int enabled;
    uint64_t generation;
} TrayCommand;

static GAsyncQueue *commands;
static GMutex commands_mutex;
static guint commands_source_id;
static GtkStatusIcon *status_icon;
static GtkWidget *menu;
static TrayEventCallback event_callback;
static void *event_user_data;
static uint64_t current_generation;
static gboolean icon_enabled;

/* Called on the GTK owner thread after gtk_main() stops or startup fails. */
static void clear_commands(void) {
    g_mutex_lock(&commands_mutex);
    GAsyncQueue *queue = commands;
    commands = NULL;
    if (commands_source_id != 0) {
        g_source_remove(commands_source_id);
        commands_source_id = 0;
    }
    if (queue != NULL) {
        TrayCommand *pending;
        while ((pending = g_async_queue_try_pop(queue)) != NULL) {
            g_free(pending);
        }
        g_async_queue_unref(queue);
    }
    g_mutex_unlock(&commands_mutex);
}

static void emit_event(int event, uint64_t generation) {
    if (event_callback != NULL) {
        event_callback(event, generation, event_user_data);
    }
}

static int startup_failed(void) {
    emit_event(EVENT_FAILED, current_generation);
    clear_commands();
    event_callback = NULL;
    event_user_data = NULL;
    return 0;
}

static void emit_embedding_state(void) {
    if (status_icon == NULL) return;
    emit_event(gtk_status_icon_is_embedded(status_icon)
                   ? EVENT_EMBEDDED
                   : EVENT_UNEMBEDDED,
               current_generation);
}

static void release_pixels(guchar *pixels, gpointer unused) {
    (void)unused;
    g_free(pixels);
}

static gboolean process_commands(gpointer unused) {
    (void)unused;
    TrayCommand *command;
    g_mutex_lock(&commands_mutex);
    commands_source_id = 0;
    g_mutex_unlock(&commands_mutex);
    while (TRUE) {
        g_mutex_lock(&commands_mutex);
        GAsyncQueue *queue = commands;
        command = queue != NULL ? g_async_queue_try_pop(queue) : NULL;
        g_mutex_unlock(&commands_mutex);
        if (command == NULL) break;
        if (command->command == COMMAND_SET_ENABLED) {
            current_generation = command->generation;
            icon_enabled = command->enabled != 0;
            if (!icon_enabled) {
                gtk_menu_popdown(GTK_MENU(menu));
            }
            gtk_status_icon_set_visible(status_icon, icon_enabled);
            emit_embedding_state();
        } else if (command->command == COMMAND_QUERY_STATE) {
            if (command->generation == current_generation && status_icon != NULL) {
                emit_embedding_state();
            }
        } else if (command->command == COMMAND_SHUTDOWN) {
            icon_enabled = FALSE;
            if (status_icon != NULL) {
                gtk_status_icon_set_visible(status_icon, FALSE);
            }
            clear_commands();
            gtk_main_quit();
            g_free(command);
            return G_SOURCE_REMOVE;
        }
        g_free(command);
    }
    return G_SOURCE_REMOVE;
}

static void enqueue_command(int command, int enabled, uint64_t generation) {
    TrayCommand *queued = g_new(TrayCommand, 1);
    queued->command = command;
    queued->enabled = enabled;
    queued->generation = generation;
    g_mutex_lock(&commands_mutex);
    if (commands != NULL) {
        g_async_queue_push(commands, queued);
        if (commands_source_id == 0) {
            commands_source_id = g_idle_add(process_commands, NULL);
        }
        queued = NULL;
    }
    g_mutex_unlock(&commands_mutex);
    g_free(queued);
}

void med_tracker_x11_tray_set_enabled(int enabled, uint64_t generation) {
    enqueue_command(COMMAND_SET_ENABLED, enabled, generation);
}

void med_tracker_x11_tray_query_state(uint64_t generation) {
    enqueue_command(COMMAND_QUERY_STATE, 0, generation);
}

void med_tracker_x11_tray_shutdown(void) {
    enqueue_command(COMMAND_SHUTDOWN, 0, 0);
}

static void on_activate(GtkStatusIcon *icon, gpointer unused) {
    (void)icon;
    (void)unused;
    if (icon_enabled) {
        emit_event(EVENT_ACTIVATE, current_generation);
    }
}

static void on_show_application(GtkMenuItem *item, gpointer unused) {
    (void)item;
    (void)unused;
    if (icon_enabled) {
        emit_event(EVENT_SHOW, current_generation);
    }
}

static void on_exit_clicked(GtkMenuItem *item, gpointer unused) {
    (void)item;
    (void)unused;
    if (icon_enabled) {
        emit_event(EVENT_EXIT, current_generation);
    }
}

static void on_popup_menu(GtkStatusIcon *icon, guint button, guint activate_time,
                          gpointer unused) {
    (void)unused;
    if (icon_enabled) {
        gtk_menu_popup(GTK_MENU(menu), NULL, NULL,
                       gtk_status_icon_position_menu, icon,
                       button, activate_time);
    }
}

static void on_embedded_changed(GObject *object, GParamSpec *property,
                                gpointer unused) {
    (void)object;
    (void)property;
    (void)unused;
    emit_embedding_state();
}

int med_tracker_x11_tray_run(TrayEventCallback callback, void *user_data,
                             const uint8_t *rgba, int width, int height,
                             int stride) {
    event_callback = callback;
    event_user_data = user_data;
    if (!gtk_init_check(NULL, NULL)) {
        return startup_failed();
    }
    if (rgba == NULL || width <= 0 || height <= 0 || stride < width * 4) {
        return startup_failed();
    }

    g_mutex_lock(&commands_mutex);
    commands = g_async_queue_new();
    g_mutex_unlock(&commands_mutex);
    guchar *pixels = g_malloc((gsize)stride * (gsize)height);
    memcpy(pixels, rgba, (gsize)stride * (gsize)height);
    GdkPixbuf *pixbuf = gdk_pixbuf_new_from_data(
        pixels, GDK_COLORSPACE_RGB, TRUE, 8, width, height, stride,
        release_pixels, NULL);
    if (pixbuf == NULL) {
        g_free(pixels);
        return startup_failed();
    }

    status_icon = gtk_status_icon_new_from_pixbuf(pixbuf);
    g_object_unref(pixbuf);
    if (status_icon == NULL) {
        return startup_failed();
    }
    gtk_status_icon_set_tooltip_text(status_icon, "Med-Tracker");
    gtk_status_icon_set_title(status_icon, "Med-Tracker");
    gtk_status_icon_set_visible(status_icon, FALSE);
    g_signal_connect(status_icon, "activate", G_CALLBACK(on_activate), NULL);
    g_signal_connect(status_icon, "popup-menu", G_CALLBACK(on_popup_menu), NULL);
    g_signal_connect(status_icon, "notify::is-embedded",
                     G_CALLBACK(on_embedded_changed), NULL);

    menu = gtk_menu_new();
    g_object_ref_sink(menu);
    GtkWidget *show = gtk_menu_item_new_with_label("Show Application");
    GtkWidget *separator = gtk_separator_menu_item_new();
    GtkWidget *exit = gtk_menu_item_new_with_label("Exit");
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), show);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), separator);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), exit);
    g_signal_connect(show, "activate", G_CALLBACK(on_show_application), NULL);
    g_signal_connect(exit, "activate", G_CALLBACK(on_exit_clicked), NULL);
    gtk_widget_show_all(menu);

    emit_event(EVENT_INITIALIZED, current_generation);
    gtk_main();

    clear_commands();
    event_callback = NULL;
    event_user_data = NULL;
    gtk_widget_destroy(menu);
    g_object_unref(menu);
    menu = NULL;
    g_signal_handlers_disconnect_by_func(status_icon,
                                        G_CALLBACK(on_embedded_changed), NULL);
    g_object_unref(status_icon);
    status_icon = NULL;
    return 1;
}
