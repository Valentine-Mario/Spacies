use crate::handlers::*;
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/verify", web::get().to(user_handler::verify_user))
            .route(
                "/resend/verification",
                web::get().to(user_handler::resend_verification),
            )
            .route("/profile", web::get().to(user_handler::get_profile))
            .route("/add", web::post().to(user_handler::add_user))
            .route("/updatename", web::post().to(user_handler::update_name))
            .route(
                "/updatepassword",
                web::post().to(user_handler::update_password),
            )
            .route("/addimg", web::post().to(user_handler::update_profile_img))
            .route(
                "/forgotpassword",
                web::post().to(user_handler::forgot_password),
            )
            .route("/login", web::post().to(user_handler::login)),
    )
    .service(
        web::scope("/spacies")
            .route("/get/{info}", web::get().to(space_handler::get_space))
            .route("/myspaces", web::get().to(space_handler::get_user_space))
            .route("/invitepage", web::get().to(space_handler::invite_page))
            .route("/leave/{info}", web::get().to(space_handler::leave_space))
            .route(
                "/getusers/{info}",
                web::get().to(space_handler::get_users_in_space),
            )
            .route(
                "/getstatus/{info}",
                web::get().to(space_handler::get_user_space_status),
            )
            .route(
                "/updateimg/{info}",
                web::post().to(space_handler::update_space_logo),
            )
            .route(
                "/updatespace/{info}",
                web::post().to(space_handler::update_space),
            )
            .route(
                "/acceptinvite",
                web::post().to(space_handler::add_invited_user),
            )
            .route(
                "changeuserstatus/{info}",
                web::post().to(space_handler::change_user_priviledge_status),
            )
            .route("/invite/{info}", web::post().to(space_handler::invite_user))
            .route(
                "/kickout/{info}",
                web::post().to(space_handler::remove_user_from_space),
            )
            .route("/add", web::post().to(space_handler::add_space)),
    )
    .service(
        web::scope("/channel")
            .route(
                "/get/{info}",
                web::get().to(channel_handler::get_channels_in_space),
            )
            .route(
                "/delete/{info}/{channel}",
                web::get().to(channel_handler::delete_channel),
            )
            .route(
                "/getdetails/{info}/{channel}",
                web::get().to(channel_handler::get_channel_details),
            )
            .route(
                "/add/{info}",
                web::post().to(channel_handler::create_new_channel),
            )
            .route(
                "/update/{info}/{channel}",
                web::post().to(channel_handler::edit_channel_name),
            ),
    )
    .service(
        web::scope("/maillist")
            .route(
                "/get/{info}",
                web::get().to(mail_folder_handler::get_space_mail_folder),
            )
            .route(
                "/getuser/{info}/{name}",
                web::get().to(mail_folder_handler::get_mail_folder_details),
            )
            .route(
                "/delete/{info}/{id}",
                web::get().to(mail_folder_handler::delete_mail_folder),
            )
            .route(
                "/add/{info}",
                web::post().to(mail_folder_handler::add_mail_folder),
            ),
    )
    .service(
        web::scope("/usermail")
            .route(
                "/add/{info}/{id}",
                web::post().to(mail_user_handler::add_user_folder),
            )
            .route(
                "/send/{info}/{id}",
                web::post().to(mail_user_handler::send_mail_to_folder),
            )
            .route(
                "/general/{info}",
                web::post().to(mail_user_handler::send_email_to_general),
            )
            .route(
                "/delete/{info}/{id}",
                web::post().to(mail_user_handler::remove_user_folder),
            ),
    )
    .service(
        web::scope("/assetfolder")
            .route(
                "/get/{info}",
                web::get().to(asset_handler::get_asset_folder),
            )
            .route(
                "/search/{info}/{name}",
                web::get().to(asset_handler::search_asset_folder),
            )
            .route(
                "/delete/{info}/{id}",
                web::get().to(asset_handler::delete_asset_folder),
            )
            .route(
                "/add/{info}",
                web::post().to(asset_handler::create_asset_folder),
            )
            .route(
                "/update/{info}/{id}",
                web::post().to(asset_handler::update_folder_name),
            ),
    )
    .service(
        web::scope("/asset")
            .route(
                "/get/{info}/{name}",
                web::get().to(asset_content_handler::get_files),
            )
            .route(
                "/delete/{info}/{id}",
                web::get().to(asset_content_handler::delete_upload),
            )
            .route(
                "/add/{info}/{id}/{file_type}",
                web::post().to(asset_content_handler::upload_file_db),
            ),
    )
    .service(
        web::scope("/event")
            .route("/get/{info}", web::get().to(event_handler::get_events))
            .route(
                "/getdetails/{info}/{channel}",
                web::get().to(event_handler::get_event_details),
            )
            .route(
                "/delete/{info}/{id}",
                web::get().to(event_handler::delete_event),
            )
            .route(
                "/search/{info}/{name}",
                web::get().to(event_handler::search_event),
            )
            .route("/add/{info}", web::post().to(event_handler::create_event))
            .route(
                "/update/{info}/{id}",
                web::post().to(event_handler::edit_event),
            ),
    );
}
