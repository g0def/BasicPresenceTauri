import { BarChart, PieChart } from "echarts/charts";
import type { BarSeriesOption, PieSeriesOption } from "echarts/charts";
import {
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import type {
  DataZoomComponentOption,
  GridComponentOption,
  LegendComponentOption,
  TooltipComponentOption,
} from "echarts/components";
import * as echarts from "echarts/core";
import type { ComposeOption } from "echarts/core";
import { LabelLayout } from "echarts/features";
import { CanvasRenderer } from "echarts/renderers";

// Tree-shakeable registration: only the bar/pie charts and the components the
// stats page actually uses are bundled, keeping ECharts' footprint down. This
// module is imported lazily (via React.lazy on the chart components) so none of
// it loads on the calendar route.
echarts.use([
  BarChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
  DataZoomComponent,
  LabelLayout,
  CanvasRenderer,
]);

/** Option type narrowed to exactly the registered charts/components — stricter
 * than the default `EChartsOption`, so a missing registration shows up as a type
 * error rather than a blank chart at runtime. */
export type ECOption = ComposeOption<
  | BarSeriesOption
  | PieSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | LegendComponentOption
  | DataZoomComponentOption
>;

export { echarts };
