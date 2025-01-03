use sfml::graphics::Drawable;
use sfml::graphics::Font;
use sfml::graphics::RenderStates;
use sfml::graphics::RenderTarget;
use sfml::graphics::Text;
use sfml::graphics::Transformable;
use sfml::system::Vector2f;
use sfml::SfBox;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct Logger {
    font: SfBox<Font>,
    messages: Vec<String>,
    buffer: String,
}

impl Logger {
    pub fn new(font: SfBox<Font>) -> Self {
        Self {
            font,
            messages: Vec::default(),
            buffer: String::default(),
        }
    }

    pub fn push(&mut self, message: String) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(&message);
        self.messages.push(message);
    }

    pub fn push_if_unique(&mut self, message: String) {
        if !self.messages.contains(&message) {
            self.push(message);
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.buffer.clear();
    }

    #[expect(dead_code)]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    #[expect(dead_code)]
    pub fn get_mut(&mut self, index: usize) -> Option<LoggerMessage<'_>> {
        (index < self.messages.len()).then(move || LoggerMessage {
            logger: self,
            index,
        })
    }

    fn update_buffer(&mut self) {
        self.buffer.clear();
        for message in &self.messages {
            if !self.buffer.is_empty() {
                self.buffer.push('\n');
            }
            self.buffer.push_str(message);
        }
    }
}

impl Drawable for Logger {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        if self.buffer.is_empty() {
            return;
        }

        let view_pos = {
            let view = target.view();
            view.center() - view.size() / 2.
        };

        let mut text = Text::new(&self.buffer, &self.font, 18);
        text.set_scale((0.5, 0.5));
        text.set_position(view_pos + Vector2f::new(5., 5.));
        text.set_outline_thickness(1.5);
        target.draw_text(&text, states);
    }
}

pub struct LoggerMessage<'l> {
    logger: &'l mut Logger,
    index: usize,
}

impl Deref for LoggerMessage<'_> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.logger.messages[self.index]
    }
}

impl DerefMut for LoggerMessage<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logger.messages[self.index]
    }
}

impl Drop for LoggerMessage<'_> {
    fn drop(&mut self) {
        self.logger.update_buffer();
    }
}
