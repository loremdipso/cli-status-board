use colored::ColoredString;

#[derive(Clone, Debug)]
pub struct Column {
    data: Vec<ColoredString>,
    original_data: Vec<ColoredString>,
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

#[derive(Clone, Copy, Debug)]
pub enum ColumnFit {
    NORMAL,
    MIN(usize),
    MAX(usize),
    EXACT(usize),
}

impl Column {
    pub fn new(config: ColumnConfig) -> Self {
        Self {
            config,
            data: Vec::new(),
            original_data: Vec::new(),
            longest: 0,
        }
    }

    pub fn push(&mut self, mut value: ColoredString) {
        self.original_data.push(value.clone());

        match self.config.fit {
            ColumnFit::EXACT(len) => {
                let len = len - self.config.left_padding - self.config.right_padding;
                if value.input.len() > len {
                    value.input = value
                        .input
                        .chars()
                        .take(len - 3)
                        .chain("...".chars())
                        .collect();
                } else if value.input.len() < len {
                    match &self.config.align {
                        ColumnAlign::LEFT => {
                            value.input = format!("{: <width$}", value.input, width = len);
                        }
                        ColumnAlign::RIGHT => {
                            value.input = format!("{: >width$}", value.input, width = len);
                        }
                    }
                }
            }
            ColumnFit::MIN(min_len) => {
                let min_len = min_len - self.config.left_padding - self.config.right_padding;
                if value.input.len() < min_len {
                    match &self.config.align {
                        ColumnAlign::LEFT => {
                            value.input = format!("{: <width$}", value.input, width = min_len);
                        }
                        ColumnAlign::RIGHT => {
                            value.input = format!("{: >width$}", value.input, width = min_len);
                        }
                    }
                }
            }
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
            ColumnFit::NORMAL => {}
        }
        self.longest = self.longest.max(value.len());
        self.data.push(value);
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

    pub fn to_wide_string(&mut self, index: usize, available_width: usize) -> String {
        if index >= self.original_data.len()
            || available_width < (self.config.left_padding + self.config.right_padding).max(3)
        {
            return String::new();
        }

        let available_width =
            available_width - self.config.left_padding - self.config.right_padding;

        let mut value = self.original_data[index].clone();
        if value.input.len() > available_width {
            value.input = value
                .input
                .chars()
                .take(available_width - 3)
                .chain("...".chars())
                .collect();
        }

        match self.config.align {
            ColumnAlign::LEFT => format!(
                "{:left_padding$}{: <width$}{:right_padding$}",
                "",
                value,
                "",
                left_padding = self.config.left_padding,
                width = available_width,
                right_padding = self.config.right_padding
            ),
            ColumnAlign::RIGHT => format!(
                "{:left_padding$}{: >width$}{:right_padding$}",
                "",
                value,
                "",
                left_padding = self.config.left_padding,
                width = available_width,
                right_padding = self.config.right_padding
            ),
        }
    }

    pub(crate) fn line_len(&self) -> usize {
        self.longest + self.config.left_padding + self.config.right_padding
    }

    pub(crate) fn is_empty(&self, row_index: usize) -> bool {
        match self.data.get(row_index) {
            Some(data) => data.len() == 0,
            None => true,
        }
    }
}
