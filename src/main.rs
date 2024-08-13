use std::sync::{mpsc};
use glow::HasContext;
use slint::SharedString;

slint::include_modules!();

mod player;
mod gl_utils;

fn main() {
    
    // let file = String::from("./example/videos/Horimiya_01.mkv"); // YUV420P10LE
    let file = String::from("./example/videos/sample_subs.mkv"); // YUV420P
    // let file = String::from("http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/TearsOfSteel.mp4"); // YUV420P

    let ictx = ffmpeg_next::format::input(&file).unwrap();
    let video_stream = ictx.streams().best(ffmpeg_next::media::Type::Video).unwrap();
    
    let codec = ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters()).unwrap();
    let video = codec.decoder().video().unwrap();
    let width = video.width();
    let height = video.height();
    let format = video.format();

    println!("Video resolution: {}x{}", width, height);
    println!("Video format: {:?}", format);


    let app = App::new().unwrap();
    let app_weak = app.as_weak();

    

    let (frame_sender, frame_receiver) = mpsc::channel::<ffmpeg_next::util::frame::Video>();
    let (subtitle_sender, subtitle_receiver) = mpsc::channel::<ffmpeg_next::Packet>();

    

    let mut video_underlay = None;

    if let Err(error) = app
        .window()
        .set_rendering_notifier({
            move |state, graphics_api| {
                match state {
                    slint::RenderingState::RenderingSetup => {
                        let gl_context = match graphics_api {
                            slint::GraphicsAPI::NativeOpenGL { get_proc_address } => unsafe {
                                glow::Context::from_loader_function_cstr(|s| get_proc_address(s))
                            },
                            _ => return,
                        };
                        video_underlay = Some(gl_utils::GLTextureYUV::new(gl_context, width, height));
                    }
                    slint::RenderingState::BeforeRendering => {
                        if let (Some(underlay), Some(app)) = (video_underlay.as_mut(), app_weak.upgrade()) {
                            // Receive the latest frame from the video thread
                            if let Ok(new_frame) = frame_receiver.try_recv() {
                                // Update the GL texture with the new frame data
                                underlay.read_video_frame(&new_frame);
                            }
                            underlay.render();
                            app.window().request_redraw();
                        }
                        if let (Some(app), Some(subtitle)) = (app_weak.upgrade(), subtitle_receiver.try_recv().ok()) {
                            app.set_subtitles_text(SharedString::from(String::from_utf8(subtitle.data().unwrap().to_vec()).unwrap()));
                        }
                    }
                    slint::RenderingState::AfterRendering => {}
                    slint::RenderingState::RenderingTeardown => {
                        drop(video_underlay.take());
                    }
                    _ => {}
                }
            }
        })
    {
        match error {
            slint::SetRenderingNotifierError::Unsupported => {
                eprintln!("This example requires the use of the GL backend. Please run with the environment variable SLINT_BACKEND=GL set.");
            }
            _ => unreachable!(),
        }
        std::process::exit(1);
    }

    let mut player = player::Player::start(
        file.into(),
        move |new_frame| {
            let _ = frame_sender.send(new_frame.clone());
        },
        move |subtitle| {
            if subtitle.data().unwrap().len() == 0 {
                return;
            }
            let _ = subtitle_sender.send(subtitle.clone());
        },
        {
            let app_weak = app.as_weak();

            move |playing| {
                app_weak.upgrade_in_event_loop(move |app| app.set_playing(playing)).unwrap();
            }
        },
    )
    .unwrap();

    app.on_toggle_pause_play(move || {
        player.toggle_pause_playing();
    });

    
    

    app.run().unwrap();
}
