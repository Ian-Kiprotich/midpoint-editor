use midpoint_engine::animations::skeleton::Joint;
use midpoint_engine::floem::common::create_icon;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::event::EventListener;
use midpoint_engine::floem::event::EventPropagation;
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::create_rw_signal;
use midpoint_engine::floem::reactive::RwSignal;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::style::CursorStyle;
use midpoint_engine::floem::taffy::AlignItems;
use midpoint_engine::floem::text::Weight;
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::svg;
use std::sync::{Arc, Mutex};

use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::views::{container, dyn_container, dyn_stack, empty, label, v_stack};
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, IntoView, View, WindowHandle};

use crate::editor_state::StateHelper;

// #[derive(Clone, Debug)]
// pub struct Joint {
//     pub id: String,
//     pub name: String,
//     pub parent_id: Option<String>,
//     pub children: Vec<String>,
//     // Add any other joint-specific properties here
//     pub position: (f32, f32), // x, y coordinates
//     pub rotation: f32,
//     pub scale: (f32, f32),
// }

// impl Joint {
//     pub fn new(name: String) -> Self {
//         Self {
//             id: String::new_v4(),
//             name,
//             parent_id: None,
//             children: Vec::new(),
//             position: (0.0, 0.0),
//             rotation: 0.0,
//             scale: (1.0, 1.0),
//         }
//     }
// }

pub fn part_properties(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let back_active = create_rw_signal(false);
    let dragger_id = create_rw_signal(String::new());
    let joints: RwSignal<Vec<Joint>> = create_rw_signal(Vec::new());

    v_stack((h_stack((
        small_button(
            "",
            "arrow-left",
            {
                move |_| {
                    println!("Click back!");
                    // this action runs on_click_stop so should stop propagation
                    // object_selected_signal.update(|v| {
                    //     *v = false;
                    // });
                    // selected_object_id_signal.update(|v| {
                    //     *v = Uuid::nil();
                    // });
                }
            },
            back_active,
        )
        .style(|s| s.margin_right(7.0)),
        label(|| "Properties").style(|s| s.font_size(24.0).font_weight(Weight::THIN)),
        joint_tree(state_helper, joints, dragger_id),
    ))
    .style(|s| s.margin_bottom(12.0)),))
    .style(|s| card_styles(s))
    .style(|s| {
        s.width(300)
            // .absolute()
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
        // .z_index(10)
    })
}

pub fn joint_tree(
    state_helper: Arc<Mutex<StateHelper>>,
    joints: RwSignal<Vec<Joint>>,
    dragger_id: RwSignal<String>,
) -> impl IntoView {
    dyn_stack(
        move || joints.get(),
        |joint: &Joint| joint.id.clone(),
        move |joint| {
            let depth = calculate_depth(&joints.get(), joint.id.clone());
            joint_item(
                state_helper.clone(),
                joints,
                dragger_id,
                joint.clone(),
                depth,
            )
        },
    )
    .style(|s| s.flex_col().column_gap(5).padding(10))
    .into_view()
}

fn calculate_depth(joints: &[Joint], joint_id: String) -> usize {
    let mut depth = 0;
    let mut current_joint = joints.iter().find(|b| b.id == joint_id);

    while let Some(joint) = current_joint {
        if let Some(parent_id) = joint.parent_id.clone() {
            depth += 1;
            current_joint = joints.iter().find(|b| b.id == parent_id);
        } else {
            break;
        }
    }
    depth
}

pub fn joint_item(
    state_helper: Arc<Mutex<StateHelper>>,
    joints: RwSignal<Vec<Joint>>,
    dragger_id: RwSignal<String>,
    joint: Joint,
    depth: usize,
) -> impl IntoView {
    let indent = depth * 20; // 20 pixels per level of depth
    let joint_id = joint.id.clone();
    let joint_name = joint.name.clone();

    h_stack((
        // Indentation spacer
        empty().style(move |s| s.width(indent as f64)),
        // Expand/collapse button (if has children)
        {
            // let has_children = !joint.children.is_empty();
            let joint_data = &mut joints.get();
            let affected_children = get_all_child_ids(joint_data, &joint_id.clone());
            if affected_children.len() > 0 {
                svg(create_icon("chevron-right"))
                    .style(|s| s.width(16).height(16).color(Color::BLACK))
                    .style(|s| s.margin_right(4.0))
                    .into_any()
            } else {
                empty().style(|s| s.width(20)).into_any() // Placeholder for alignment
            }
        },
        // Joint icon
        svg(create_icon("joint"))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0))
            .on_event_stop(EventListener::PointerDown, |_| {}),
        // Joint name
        label({
            let joint_name = joint_name.clone();

            move || joint_name.clone().to_string()
        })
        .style(|s| s.selectable(false).cursor(CursorStyle::RowResize)),
    ))
    .style(|s| s.selectable(false).cursor(CursorStyle::RowResize))
    .draggable()
    .on_event(EventListener::DragStart, {
        let joint_id = joint_id.clone();

        move |_| {
            dragger_id.set(joint_id.clone());
            EventPropagation::Continue
        }
    })
    .on_event(EventListener::DragOver, {
        let joint_id = joint_id.clone();

        move |_| {
            // let mut editor = editor.lock().unwrap();
            let dragger_id = dragger_id.get_untracked();
            let joint_id = joint_id.clone();

            if dragger_id != joint_id {
                // joints.update(|joints| {
                let joint_data = joints.get();
                let new_data = &mut joint_data.clone();
                if let (Some(dragger_pos), Some(hover_pos)) = (
                    joint_data.iter().position(|j| j.id == dragger_id),
                    joint_data.iter().position(|j| j.id == joint_id),
                ) {
                    // Prevent cycles in the joint hierarchy
                    if !would_create_cycle(new_data, &dragger_id, &joint_id) {
                        // Store the dragged joint and its children's IDs
                        let dragged_joint = new_data[dragger_pos].clone();
                        let affected_children = get_all_child_ids(new_data, &dragged_joint.id);

                        // Create updated version of dragged joint with new parent
                        let mut updated_joint = dragged_joint.clone();
                        updated_joint.parent_id = Some(joint_id);

                        // Update local position relative to new parent
                        updated_joint.local_position =
                            calculate_new_local_position(new_data, &updated_joint, &joint);

                        // Remove and reinsert the joint at new position

                        new_data.remove(dragger_pos);
                        new_data.insert(hover_pos, updated_joint);

                        // Update positions of all affected children
                        // do we really want to move bones when changing order or hierarchy?
                        // update_child_transforms(new_data, &affected_children);

                        joints.set(new_data.to_vec());

                        // Update the editor's joint hierarchy
                        // editor.update_joint_hierarchy(joints);
                    }
                }
                // });
            }
            EventPropagation::Continue
        }
    })
    .dragging_style(|s| {
        s.box_shadow_blur(3)
            .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
            .box_shadow_spread(2)
    })
    .style(|s| {
        s.width(220.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .padding_vert(8)
            .background(Color::rgb(255.0, 239.0, 194.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| s.background(Color::rgb(222.0, 206.0, 160.0)))
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
}

fn would_create_cycle(joints: &mut Vec<Joint>, dragged_id: &str, new_parent_id: &str) -> bool {
    let mut current_id: Option<_> = Some(new_parent_id);
    while let Some(id) = current_id {
        if id == dragged_id {
            return true;
        }
        current_id = joints
            .iter()
            .find(|j| j.id == id)
            .and_then(|j| j.parent_id.as_deref());
    }
    false
}

fn get_all_child_ids(joints: &mut Vec<Joint>, parent_id: &str) -> Vec<String> {
    let mut children = Vec::new();
    let mut to_process = vec![parent_id];

    while let Some(current_id) = to_process.pop() {
        for joint in joints.iter() {
            if joint.parent_id.as_deref() == Some(current_id) {
                children.push(joint.id.clone());
                to_process.push(&joint.id);
            }
        }
    }

    children
}

fn calculate_new_local_position(
    joints: &mut Vec<Joint>,
    child: &Joint,
    new_parent: &Joint,
) -> [f32; 3] {
    // Calculate the difference between world positions
    // This is a simplified example - you'll need to implement proper
    // transformation math based on your coordinate system
    let child_world_pos = get_world_position(joints, child);
    let parent_world_pos = get_world_position(joints, new_parent);

    [
        child_world_pos[0] - parent_world_pos[0],
        child_world_pos[1] - parent_world_pos[1],
        child_world_pos[2] - parent_world_pos[2],
    ]
}

fn get_world_position(joints: &mut Vec<Joint>, joint: &Joint) -> [f32; 3] {
    let mut world_pos = joint.local_position;
    let mut current_joint = joint;

    // Walk up the hierarchy accumulating transforms
    while let Some(parent_id) = &current_joint.parent_id {
        if let Some(parent) = joints.iter().find(|j| &j.id == parent_id) {
            // Add parent's position to accumulate world position
            // Note: This is simplified - you should properly apply orientation/scale
            world_pos[0] += parent.local_position[0];
            world_pos[1] += parent.local_position[1];
            world_pos[2] += parent.local_position[2];
            current_joint = parent;
        } else {
            break;
        }
    }

    world_pos
}

// fn update_child_transforms(joints: &mut Vec<Joint>, child_ids: &[String]) {
//     // Update the local transforms of all affected children
//     // This is important when a parent joint moves
//     for child_id in child_ids {
//         if let Some(child) = joints.iter().find(|j| &j.id == child_id) {
//             // let child = &joints[child_idx];
//             // let child = joints.get_mut(child_idx).expect("Couldn't get child");
//             if let Some(parent_id) = &child.parent_id {
//                 if let Some(parent) = joints.iter().find(|j| &j.id == parent_id) {
//                     let new_local_pos = calculate_new_local_position(joints, child, parent);
//                     // joints[child_idx].local_position = new_local_pos;
//                     child.set_local_position(new_local_pos);
//                 }
//             }
//         }
//     }
// }