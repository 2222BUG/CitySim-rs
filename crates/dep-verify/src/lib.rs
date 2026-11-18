//! dep-verify —— 2.1 技术基线版本共存验证（Stage 0 Step 8）。
//!
//! 断言（主文档 2.1）：Bevy 0.18.1、Rust 1.89、bevy_egui 0.39.x、rapier 0.33.x。
//! 本 crate 的 `cargo check` / `cargo +1.89 check` 通过即为该基线的复现验证证据，
//! 解析后的精确版本记录于 Cargo.lock 与 docs/benchmarks/dep-verify-REPORT.md。
//!
//! 口径：Rapier 以 f32 为基础（rapier2d/rapier3d 默认 Real=f32），按 ADR-NEW-15
//! 只能用于非权威侧或经 ADR 豁免；本 crate 仅验证版本共存，不涉及权威物理。
#![forbid(unsafe_code)]

// 引用符号确保四个依赖全部参与链接与编译。
pub use bevy;
pub use bevy_egui;
pub use rapier2d;
pub use rapier3d;

/// 基线声明（与 2.1 对照）。
pub fn baseline() -> (&'static str, &'static str, &'static str, &'static str) {
    ("bevy 0.18.1", "rustc 1.89", "bevy_egui 0.39.x", "rapier 0.33.x")
}

#[cfg(test)]
mod tests {
    #[test]
    fn baseline_strings_match_spec() {
        let (b, r, e, p) = super::baseline();
        assert!(b.starts_with("bevy 0.18.1"));
        assert!(r.starts_with("rustc 1.89"));
        assert!(e.starts_with("bevy_egui 0.39"));
        assert!(p.starts_with("rapier 0.33"));
    }

    /// 关键 API 验证（2.1"正式锁定前必须完成"第 4 项）：
    /// Bevy ECS 调度、bevy_egui 插件类型、Rapier 2d/3d 构建器均可用且签名如预期。
    #[test]
    fn key_apis_are_usable() {
        // Bevy：App + Resource + 系统调度（2.9 条 5 显式顺序的前提）
        use bevy::prelude::*;
        #[derive(Resource)]
        struct Counter(u32);
        fn tick(mut c: ResMut<Counter>) {
            c.0 += 1;
        }
        let mut app = App::new();
        app.insert_resource(Counter(0));
        app.add_systems(Update, tick);
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Counter>().0, 2, "two updates = two runs");

        // bevy_egui：插件与 SystemParam 类型可解析（构造需渲染管线，留集成验证）
        let _ = std::any::type_name::<bevy_egui::EguiPlugin>();
        let _ = std::any::type_name::<bevy_egui::EguiContexts>();

        // Rapier 2d/3d：刚体与碰撞体构建器可用（Real=f32，仅非权威侧口径）
        let body = rapier3d::dynamics::RigidBodyBuilder::dynamic().build();
        let ball = rapier2d::geometry::ColliderBuilder::ball(0.5).build();
        assert_eq!(body.body_type(), rapier3d::dynamics::RigidBodyType::Dynamic);
        assert!(ball.shape().as_ball().is_some_and(|b| (b.radius - 0.5).abs() < 1e-6));
    }
}
