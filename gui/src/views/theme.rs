use super::style;
use crate::states::SignalState;
use druid::widget::prelude::*;
pub struct ThemeScope<W> {
    inner: W,
    cached_env: Option<Env>,
}

impl<W> ThemeScope<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            cached_env: None,
        }
    }

    fn set_env(&mut self, data: &SignalState, outer_env: &Env) {
        let mut themed_env = outer_env.clone();
        style::setup(&mut themed_env, data);
        self.cached_env.replace(themed_env);
    }
}

impl<W: Widget<SignalState>> Widget<SignalState> for ThemeScope<W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SignalState, env: &Env) {
        self.inner
            .event(ctx, event, data, self.cached_env.as_ref().unwrap_or(env))
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &SignalState,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = &event {
            self.set_env(data, env);
        }
        self.inner
            .lifecycle(ctx, event, data, self.cached_env.as_ref().unwrap_or(env))
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &SignalState,
        data: &SignalState,
        env: &Env,
    ) {
        if !data.theme.same(&old_data.theme) {
            self.set_env(data, env);
            ctx.request_layout();
            ctx.request_paint();
        }
        self.inner
            .update(ctx, old_data, data, self.cached_env.as_ref().unwrap_or(env));
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &SignalState,
        env: &Env,
    ) -> Size {
        self.inner
            .layout(ctx, bc, data, self.cached_env.as_ref().unwrap_or(env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SignalState, env: &Env) {
        self.inner
            .paint(ctx, data, self.cached_env.as_ref().unwrap_or(env));
    }
}
