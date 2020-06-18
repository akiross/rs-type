use ggez::nalgebra as na;
use ggez::{graphics, Context, GameResult};

pub struct Player {
    pub pos_x: f32,
    pub pos_y: f32,
    pub name: String,
    pub font_size: f32,
}

impl Player {
    pub fn draw(&self, ctx: &mut Context, color: graphics::Color) -> GameResult<()> {
        let frag = graphics::TextFragment::new(self.name.clone());
        let frag = frag.scale(graphics::Scale::uniform(self.font_size));
        let player = graphics::Text::new(frag);
        let (w, h) = player.dimensions(ctx);
        let mesh = graphics::MeshBuilder::new()
            .circle(
                graphics::DrawMode::fill(),
                na::Point2::new(self.pos_x, self.pos_y),
                h as f32 * 0.5,
                1.0,
                color,
            )
            .build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        let params = graphics::DrawParam::default()
            .dest(na::Point2::new(
                self.pos_x - w as f32 * 0.5,
                self.pos_y + 0.5 * h as f32,
            ))
            .color(color);
        graphics::draw(ctx, &player, params)?;
        /*
         * This draws the current text, but it kinda sucks... This could show
         * the errors made in this level by the user, as alternative.
        if !self.word.is_empty() {
            let frag = graphics::TextFragment::new(self.word.clone());
            let frag = frag.scale(graphics::Scale::uniform(self.font_size));
            let text = graphics::Text::new(frag);
            graphics::draw(
                ctx,
                &text,
                graphics::DrawParam::default().dest(na::Point2::new(
                    self.pos_x - w as f32 * 0.5,
                    self.pos_y + 1.5 * h as f32,
                )),
            )?;
        }
        */
        Ok(())
    }
}

pub struct Enemy {
    pub pos_x: f32,
    pub pos_y: f32,
    pub speed: f32,
    pub word: String,
    pub font_size: f32,
}

impl Enemy {
    pub fn draw(&self, ctx: &mut Context, color: graphics::Color) -> GameResult<()> {
        let frag = graphics::TextFragment::new(self.word.clone());
        let frag = frag.scale(graphics::Scale::uniform(self.font_size));
        let text = graphics::Text::new(frag);
        let (w, h) = text.dimensions(ctx);
        let mesh = graphics::MeshBuilder::new()
            .circle(
                graphics::DrawMode::fill(),
                na::Point2::new(self.pos_x, self.pos_y),
                h as f32 * 0.5,
                1.0,
                color,
            )
            .build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        let params = graphics::DrawParam::default()
            .dest(na::Point2::new(
                self.pos_x - w as f32 * 0.5,
                self.pos_y - 1.5 * h as f32,
            ))
            .color(color);
        graphics::draw(ctx, &text, params)?;
        Ok(())
    }
}
