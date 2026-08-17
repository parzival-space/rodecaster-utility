

#[derive(Debug, Default)]
pub struct App {}

// impl eframe::App for App {
//     fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
//         ui.vertical(|ui| {
//             ui.set_height(ui.available_height());
//             ui.set_width(ui.available_width());
//
//             // background gradient
//             let rect = ui.max_rect();
//             let start_color = Color32::from_rgb(0x2d, 0x2d, 0x2d);
//             let end_color = Color32::from_rgb(0x1a, 0x1a, 0x1a);
//             let steps = 16usize.max(1);
//             for i in 0..steps {
//                 let t0 = i as f32 / steps as f32;
//                 let t1 = (i + 1) as f32 / steps as f32;
//
//                 let y0 = rect.top() + rect.height() * t0;
//                 let y1 = rect.top() + rect.height() * t1;
//
//                 let lerp = |a: u8, b: u8| -> u8 {
//                     ((a as f32) + (b as f32 - a as f32) * t0).round() as u8
//                 };
//
//                 let color = Color32::from_rgba_unmultiplied(
//                     lerp(start_color.r(), end_color.r()),
//                     lerp(start_color.g(), end_color.g()),
//                     lerp(start_color.b(), end_color.b()),
//                     255,
//                 );
//
//                 ui.painter().rect_filled(Rect::from_min_max(
//                     Pos2::new(0.0, y0),
//                     Pos2::new(rect.width(), y1),
//                 ), 0.0, color);
//             }
//
//             // device list
//             egui::Frame::NONE
//                 .fill(Color32::from_rgb(0x2d, 0x2d, 0x2d))
//                 .show(ui, |ui| {
//                     ui.set_height(ui.available_height());
//                     ui.set_width(311.0);
//
//                     ScrollArea::vertical().show(ui, |ui| {
//                         ui.set_height(ui.available_height());
//                         ui.set_width(ui.available_width());
//
//                         for i in 0..100 {
//                             ui.label(format!("Device {}", i));
//                             ui.add_space(10.0);
//                         }
//                     });
//                 });
//
//             // device settings
//
//         });
//     }
// }