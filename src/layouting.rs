use std::fmt::Write;

use crate::graph::{Graph, Node, SortOrder};

pub struct Style {
    pub top_level_margin: i32,

    pub box_width:           i32,
    pub width_between_boxes: i32,

    pub box_height:           i32,
    pub height_between_boxes: i32,

    pub margin_width:  i32,
    pub margin_height: i32,

    pub text_font_size_normal: i32,
    pub text_font_size_larger: i32,
}

impl Style {
    pub fn width_per_level(&self) -> i32 {
        self.box_width + self.width_between_boxes
    }

    pub fn height_per_level(&self) -> i32 {
        self.box_height + self.height_between_boxes
    }
}

pub const DEFAULT_STYLE: Style = Style {
    top_level_margin:      5,
    box_width:             160,
    width_between_boxes:   50,
    box_height:            40,
    height_between_boxes:  40,
    margin_width:          10,
    margin_height:         10,
    text_font_size_normal: 10,
    text_font_size_larger: 12,
};

fn draw_box(result: &mut String, node: &Node, style: &Style, x_center: i32, y_center: i32) {
    let text_vertical_offset = style.box_height / 4;

    writeln!(
        result,
        "<rect x=\"{}\" y=\"{}\" height=\"{}\" width=\"{}\" fill=\"none\" stroke=\"black\" rx=\"2\" stroke-widht=\"2\"/>",
        x_center - style.box_width / 2,
        y_center - style.box_height / 2,
        style.box_height,
        style.box_width
    )
    .unwrap();

    if let Some(op_name) = &node.op {
        writeln!(
            result,
            "<text font-size=\"{}\" font-family=\"monospace\" x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" text-anchor=\"middle\">{}</text>",
            style.text_font_size_larger,
            x_center,
            y_center - text_vertical_offset,
            op_name
        )
        .unwrap();
    }

    writeln!(
        result,
        "<text font-size=\"{}\" font-family=\"monospace\" x=\"{}\" y=\"{}\" dominant-baseline=\"middle\" text-anchor=\"middle\">{}</text>",
        style.text_font_size_normal,
        x_center,
        y_center + text_vertical_offset,
        node.name
    )
    .unwrap();
}

pub fn to_svg(g: &Graph, sort_order: &SortOrder, style: &Style) -> String {
    let level_count = sort_order.nodes_in_level.len() as i32;
    let max_count_in_single_level = *sort_order.nodes_in_level.iter().max().expect("Ther must be a max value!") as i32;

    let width = style.width_per_level() * level_count + 2 * style.margin_width;
    let height = style.height_per_level() * max_count_in_single_level + 2 * style.margin_height;

    let mut result = String::new();

    writeln!(
        &mut result,
        "<svg height=\"{}\" width=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        height, width
    )
    .unwrap();

    // It is very important to have nice formating and not have blank lines
    // This is due to markdown handling of svg content
    write!(
        &mut result,
        r#"<defs>
    <marker
        id="arrowhead"
        markerWidth="6"
        markerHeight="6"
        refX="0"
        refY="3"
        orient="auto"
    >
        <polygon points="0 0, 6 3, 0 6" />
    </marker>
</defs>
"#
    )
    .unwrap();

    writeln!(
        &mut result,
        "<rect x=\"{}\" y=\"{}\" height=\"{}\" width=\"{}\" fill=\"white\" stroke=\"black\" rx=\"2\" stroke-width=\"2\"/>",
        style.top_level_margin,
        style.top_level_margin,
        height - style.top_level_margin,
        width - style.top_level_margin,
    )
    .unwrap();

    for (i, &node_index) in sort_order.order_indices.iter().enumerate() {
        let level_index = sort_order.depths[i];
        let index_within_level = sort_order.index_at_depth[i];

        let x = (level_count - level_index as i32 - 1) * style.width_per_level() + style.margin_height + style.width_per_level() / 2;
        let y = (index_within_level as i32) * style.height_per_level()
            + style.margin_height
            + style.height_per_level() / 2
            + style.height_per_level() / 2 * (max_count_in_single_level - sort_order.nodes_in_level[level_index] as i32);

        draw_box(&mut result, g.node(node_index), style, x, y);

        for connection_index in 0..g.to_connections()[node_index].len() {
            let from_index = g.to_connections()[node_index][connection_index].from_index;

            // all depths info is in sort order
            let from_index_in_sort_order = sort_order.order_indices.iter().position(|&x| x == from_index).unwrap();

            let from_level_index = sort_order.depths[from_index_in_sort_order];
            let num_inputs = g.to_connections()[node_index].len() as i32;

            let x_from = (level_count - from_level_index as i32 - 1) * style.width_per_level()
                + style.margin_height
                + style.width_per_level() / 2
                + style.box_width / 2;

            let y_from = (sort_order.index_at_depth[from_index_in_sort_order] as i32) * style.height_per_level()
                + style.margin_height
                + style.height_per_level() / 2
                + style.height_per_level() / 2 * (max_count_in_single_level - sort_order.nodes_in_level[from_level_index] as i32);

            let y_to = y + style.box_height / 2 - (style.box_height / (num_inputs + 1)) * (num_inputs - connection_index as i32);

            let x_to = x - style.box_width / 2 - 10;

            let control_point_ext = style.width_between_boxes / 4 + (from_level_index - level_index - 1) as i32 * style.width_between_boxes;

            writeln!(
                &mut result,
                "<path d=\"M {} {} C {} {}, {} {}, {} {}\" stroke=\"black\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\" fill=\"none\"/>",
                x_from,
                y_from,
                x_from + control_point_ext,
                y_from,
                x_to - control_point_ext,
                y_to,
                x_to,
                y_to
            )
            .unwrap();
        }
    }

    writeln!(&mut result, "</svg>").unwrap();

    result
}
