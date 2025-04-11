use colored::ColoredString;

#[derive(Clone, Debug)]
pub struct Column {
    data: Vec<ColoredString>,
    config: ColumnConfig,
    pub longest: usize,
}

#[derive(Clone, Debug)]
pub struct ColumnConfig {
    pub align: ColumnAlign,
    pub fit: ColumnFit,
    pub left_padding: usize,
    pub right_padding: usize,
}

#[derive(Clone, Debug)]
pub enum ColumnAlign {
    LEFT,
    RIGHT,
}

#[derive(Clone, Debug)]
pub enum ColumnFit {
    NORMAL,
    // MIN(usize),
    MAX(usize),
    // GROW,
}

impl Column {
    pub fn new(config: ColumnConfig) -> Self {
        Self {
            config,
            data: Vec::new(),
            longest: 0,
        }
    }

    pub fn push(&mut self, mut value: ColoredString) {
        match self.config.fit {
            ColumnFit::MAX(max_len) => {
                let max_len = max_len - self.config.left_padding - self.config.right_padding;
                if value.input.len() > max_len {
                    value.input = value
                        .input
                        .chars()
                        .take(max_len - 3)
                        .chain("...".chars())
                        .collect();
                }
            }
            _ => {}
        }
        self.longest = self.longest.max(value.len());
        self.data.push(value);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn to_string(&mut self, index: usize) -> String {
        if index >= self.data.len() {
            return String::new();
        }

        match self.config.align {
            ColumnAlign::LEFT => format!(
                "{:left_padding$}{: <width$}{:right_padding$}",
                "",
                self.data[index],
                "",
                left_padding = self.config.left_padding,
                width = self.longest,
                right_padding = self.config.right_padding
            ),
            ColumnAlign::RIGHT => format!(
                "{:left_padding$}{: >width$}{:right_padding$}",
                "",
                self.data[index],
                "",
                left_padding = self.config.left_padding,
                width = self.longest,
                right_padding = self.config.right_padding
            ),
        }
    }
}
