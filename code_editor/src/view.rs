use {
    crate::{line, token::TokenInfo, Bias, Line, Selection, Settings, Text, Tokenizer},
    std::slice,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View<'a> {
    settings: &'a Settings,
    text: &'a Text,
    tokenizer: &'a Tokenizer,
    inline_text_inlays: &'a [Vec<(usize, String)>],
    inline_widget_inlays: &'a [Vec<((usize, Bias), line::Widget)>],
    soft_breaks: &'a [Vec<usize>],
    start_column_after_wrap: &'a [usize],
    fold_column: &'a [usize],
    scale: &'a [f64],
    block_widget_inlays: &'a [((usize, Bias), Widget)],
    summed_heights: &'a [f64],
    selections: &'a [Selection],
    latest_selection_index: usize,
}

impl<'a> View<'a> {
    pub fn new(
        settings: &'a Settings,
        text: &'a Text,
        tokenizer: &'a Tokenizer,
        inline_text_inlays: &'a [Vec<(usize, String)>],
        inline_widget_inlays: &'a [Vec<((usize, Bias), line::Widget)>],
        soft_breaks: &'a [Vec<usize>],
        start_column_after_wrap: &'a [usize],
        fold_column: &'a [usize],
        scale: &'a [f64],
        block_widget_inlays: &'a [((usize, Bias), Widget)],
        summed_heights: &'a [f64],
        selections: &'a [Selection],
        latest_selection_index: usize,
    ) -> Self {
        Self {
            settings,
            text,
            tokenizer,
            inline_text_inlays,
            inline_widget_inlays,
            soft_breaks,
            start_column_after_wrap,
            fold_column,
            scale,
            block_widget_inlays,
            summed_heights,
            selections,
            latest_selection_index,
        }
    }

    pub fn compute_width(&self) -> f64 {
        let mut max_width = 0.0f64;
        for element in self.elements(0, self.line_count()) {
            max_width = max_width.max(match element {
                Element::Line(_, line) => line.compute_scaled_width(),
                Element::Widget(_, widget) => widget.width,
            });
        }
        max_width
    }

    pub fn height(&self) -> f64 {
        self.summed_heights[self.line_count() - 1]
    }

    pub fn find_first_line_ending_after_y(&self, y: f64) -> usize {
        match self
            .summed_heights
            .binary_search_by(|summed_height| summed_height.partial_cmp(&y).unwrap())
        {
            Ok(line_index) => line_index + 1,
            Err(line_index) => line_index,
        }
    }

    pub fn find_first_line_starting_after_y(&self, y: f64) -> usize {
        match self
            .summed_heights
            .binary_search_by(|summed_height| summed_height.partial_cmp(&y).unwrap())
        {
            Ok(line_index) => line_index + 1,
            Err(line_index) => {
                if line_index == self.line_count() {
                    line_index
                } else {
                    line_index + 1
                }
            }
        }
    }

    pub fn settings(&self) -> &'a Settings {
        self.settings
    }

    pub fn text(self) -> &'a Text {
        &self.text
    }

    pub fn line_count(&self) -> usize {
        self.text.as_lines().len()
    }

    pub fn line(&self, line: usize) -> Line<'a> {
        Line::new(
            &self.settings,
            &self.text.as_lines()[line],
            &self.tokenizer.token_infos()[line],
            &self.inline_text_inlays[line],
            &self.inline_widget_inlays[line],
            &self.soft_breaks[line],
            self.start_column_after_wrap[line],
            self.fold_column[line],
            self.scale[line],
        )
    }

    pub fn lines(&self, start_line: usize, end_line: usize) -> Lines<'a> {
        Lines {
            settings: self.settings,
            text: self.text.as_lines()[start_line..end_line].iter(),
            token_infos: self.tokenizer.token_infos()[start_line..end_line].iter(),
            inline_text_inlays: self.inline_text_inlays[start_line..end_line].iter(),
            inline_widget_inlays: self.inline_widget_inlays[start_line..end_line].iter(),
            soft_breaks: self.soft_breaks[start_line..end_line].iter(),
            start_column_after_wrap: self.start_column_after_wrap[start_line..end_line].iter(),
            fold_column: self.fold_column[start_line..end_line].iter(),
            scale: self.scale[start_line..end_line].iter(),
        }
    }

    pub fn line_y(&self, line: usize) -> f64 {
        if line == 0 {
            0.0
        } else {
            self.summed_heights[line - 1]
        }
    }

    pub fn elements(&self, start_line: usize, end_line: usize) -> BlockElements<'a> {
        BlockElements {
            lines: self.lines(start_line, end_line),
            block_widget_inlays: &self.block_widget_inlays[self
                .block_widget_inlays
                .iter()
                .position(|((line, _), _)| *line >= start_line)
                .unwrap_or(self.block_widget_inlays.len())..],
            line: start_line,
        }
    }

    pub fn selections(&self) -> &'a [Selection] {
        self.selections
    }

    pub fn latest_selection_index(&self) -> usize {
        self.latest_selection_index
    }
}

#[derive(Clone, Debug)]
pub struct Lines<'a> {
    settings: &'a Settings,
    text: slice::Iter<'a, String>,
    token_infos: slice::Iter<'a, Vec<TokenInfo>>,
    inline_text_inlays: slice::Iter<'a, Vec<(usize, String)>>,
    inline_widget_inlays: slice::Iter<'a, Vec<((usize, Bias), line::Widget)>>,
    soft_breaks: slice::Iter<'a, Vec<usize>>,
    start_column_after_wrap: slice::Iter<'a, usize>,
    fold_column: slice::Iter<'a, usize>,
    scale: slice::Iter<'a, f64>,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Line::new(
            &self.settings,
            self.text.next()?,
            self.token_infos.next()?,
            self.inline_text_inlays.next()?,
            self.inline_widget_inlays.next()?,
            self.soft_breaks.next()?,
            *self.start_column_after_wrap.next()?,
            *self.fold_column.next()?,
            *self.scale.next()?,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct BlockElements<'a> {
    lines: Lines<'a>,
    block_widget_inlays: &'a [((usize, Bias), Widget)],
    line: usize,
}

impl<'a> Iterator for BlockElements<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self
            .block_widget_inlays
            .first()
            .map_or(false, |((line, bias), _)| {
                *line == self.line && *bias == Bias::Before
            })
        {
            let ((_, widget), block_widget_inlays) =
                self.block_widget_inlays.split_first().unwrap();
            self.block_widget_inlays = block_widget_inlays;
            return Some(Element::Widget(Bias::Before, *widget));
        }
        if self
            .block_widget_inlays
            .first()
            .map_or(false, |((line, bias), _)| {
                *line == self.line && *bias == Bias::After
            })
        {
            let ((_, widget), block_widget_inlays) =
                self.block_widget_inlays.split_first().unwrap();
            self.block_widget_inlays = block_widget_inlays;
            return Some(Element::Widget(Bias::After, *widget));
        }
        let line_ref = self.lines.next()?;
        self.line += 1;
        Some(Element::Line(false, line_ref))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Element<'a> {
    Line(bool, Line<'a>),
    Widget(Bias, Widget),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Widget {
    pub id: usize,
    pub width: f64,
    pub height: f64,
}

impl Widget {
    pub fn new(id: usize, width: f64, height: f64) -> Self {
        Self { id, width, height }
    }
}