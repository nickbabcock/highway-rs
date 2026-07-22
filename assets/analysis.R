# Analysis of the HighwayHash comparison benchmarks.
#
# Regenerate the input data with (see the README for details):

library(tidyverse)
library(scales)
library(ggnewscale)
library(RColorBrewer)

# The SIMD (and portable) HighwayHash implementations we want to visually
# distinguish from the other hash functions we benchmark against.
highway_fns <- c("avx", "sse", "portable")

df <- read_csv("./highway.csv", show_col_types = FALSE) |>
  rename(fn = `function`) |>
  mutate(
    highwayhash = fn %in% highway_fns,
    line = if_else(highwayhash, "highwayhash", "other"),
    family = factor(
      if_else(highwayhash, "HighwayHash", "Other hashes"),
      levels = c("HighwayHash", "Other hashes")
    ),
    # sample_measured_value is the total nanoseconds spent hashing
    # `iteration_count` payloads of `value` bytes each.
    throughput = value * iteration_count * 1e9 / sample_measured_value,
    hashes_per_ms = iteration_count * 1e6 / sample_measured_value
  )

df64 <- filter(df, group == "64bit")
df256 <- filter(df, group == "256bit")

# We build a custom palette so that hashes producing both a 64bit and a 256bit
# result keep a consistent color across graphs. Hashes unique to one output size
# are free to reuse colors, which keeps each individual palette small and the
# graphs easier to read.
names64 <- sort(unique(df64$fn))
names256 <- sort(unique(df256$fn))
in_both <- intersect(names64, names256)
base_palette <- brewer.pal(max(length(names64), length(names256), 3), "Set1")

make_palette <- function(fns) {
  ordered <- c(in_both, setdiff(fns, in_both))
  set_names(base_palette[seq_along(ordered)], ordered)
}

pal64 <- make_palette(names64)
pal256 <- make_palette(names256)

byte_rate <- function(x) paste0(label_bytes(units = "GB")(x), "/s")

payload_breaks <- c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)

# A cleaner, modern base theme shared across every plot.
theme_set(theme_minimal(base_size = 12))
theme_update(
  plot.title.position = "plot",
  plot.caption = element_text(hjust = 0, color = "grey40"),
  panel.grid.minor = element_blank(),
  legend.position = "right"
)

# HighwayHash implementations are drawn as solid lines and everything else as
# dashed lines. The plots otherwise share the same structure, so a single helper
# builds each one: points for every sample mean, then the mean line split into a
# "HighwayHash" and an "Other Hashes" legend via ggnewscale.
throughput_plot <- function(data, palette, y, y_label, y_format, title) {
  y <- enquo(y)
  highway <- filter(data, highwayhash)
  other <- filter(data, !highwayhash)

  ggplot(mapping = aes(value, !!y)) +
    stat_summary(
      data = data, aes(color = fn),
      fun = mean, geom = "point", size = 1.5, alpha = 0.8
    ) +
    scale_color_manual(values = palette, guide = "none") +
    new_scale_color() +
    stat_summary(
      data = highway, aes(color = fn, linetype = line),
      fun = mean, geom = "line", linewidth = 1.2
    ) +
    scale_color_manual("HighwayHash", values = palette,
                       guide = guide_legend(order = 1)) +
    scale_linetype_manual(values = c(highwayhash = "solid", other = "22"),
                          guide = "none") +
    new_scale_color() +
    stat_summary(
      data = other, aes(color = fn, linetype = line),
      fun = mean, geom = "line", linewidth = 1.2
    ) +
    scale_color_manual("Other Hashes", values = palette,
                       guide = guide_legend(order = 2)) +
    scale_y_continuous(labels = y_format, limits = c(0, NA),
                       breaks = pretty_breaks(10)) +
    scale_x_continuous(transform = "log2", limits = c(1, NA),
                       breaks = payload_breaks) +
    labs(
      title = title,
      subtitle = "Mean of criterion samples; higher is better",
      caption = "Solid lines are HighwayHash implementations",
      y = y_label,
      x = "Payload length in bytes (log2 scale)"
    )
}

throughput_plot(
  df64, pal64, throughput, "Throughput", byte_rate,
  "Throughput of 64bit hash functions at varying payload lengths"
)
ggsave("64bit-highwayhash.png", width = 8, height = 5, dpi = 100)

throughput_plot(
  df256, pal256, throughput, "Throughput", byte_rate,
  "Throughput of 256bit hash functions at varying payload lengths"
)
ggsave("256bit-highwayhash.png", width = 8, height = 5, dpi = 100)

throughput_plot(
  df256, pal256, hashes_per_ms, "Hashes per ms", label_comma(),
  "Hash rate of 256bit hash functions at varying payload lengths"
)
ggsave("256bit-highwayhash-rate.png", width = 8, height = 5, dpi = 100)

# How much does the wider 256bit output cost HighwayHash relative to its 64bit
# output? Overlaying the two makes the (small) gap easy to read.
df |>
  filter(highwayhash) |>
  ggplot(aes(value, throughput, color = fn)) +
  stat_summary(aes(linetype = group), fun = mean, geom = "line", linewidth = 1.2) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA),
                     breaks = pretty_breaks(10)) +
  scale_x_continuous(transform = "log2", limits = c(1, NA),
                     breaks = payload_breaks) +
  scale_color_manual(values = pal256) +
  labs(
    title = "Throughput of 64bit vs 256bit HighwayHash",
    col = "HighwayHash",
    linetype = "Output",
    y = "Throughput",
    x = "Payload length in bytes (log2 scale)"
  )
ggsave("64bit-vs-256bit-highwayhash.png", width = 8, height = 5, dpi = 100)

# A heatmap of mean throughput. Shading is relative within each (payload, output
# size) cell so the fastest hash for every workload stands out, while the printed
# GB/s number keeps the absolute value readable.
#
# Hash libraries go on the x axis and payload size on the y axis. Family is the
# column facet so the HighwayHash strip sits above a column spanning both output
# size rows, making it clear that avx / portable / sse produce a 64bit *and* a
# 256bit hash while the third party hashes only ever do one or the other.
# Within a family the hashes are ordered by the output size they support, so the
# 256bit-only and 64bit-only libraries form contiguous blocks rather than being
# scattered among the cells they have no result for.
reldf <- df |>
  mutate(throughput = throughput / 1e9) |>
  group_by(group, family, fn, value) |>
  summarize(throughput = mean(throughput), .groups = "drop") |>
  group_by(group, value) |>
  mutate(relative = throughput / max(throughput)) |>
  ungroup()

fn_order <- reldf |>
  group_by(family, fn) |>
  summarize(supports = paste(sort(unique(group)), collapse = "+"), .groups = "drop") |>
  arrange(family, supports, fn) |>
  pull(fn)
reldf$fn <- factor(reldf$fn, levels = fn_order)

ggplot(reldf, aes(fn, as.factor(value))) +
  geom_tile(aes(fill = relative), color = "white") +
  geom_text(
    aes(label = format(round(throughput, 2), nsmall = 2), color = relative > 0.55),
    size = 3.25
  ) +
  facet_grid(
    rows = vars(group), cols = vars(family),
    scales = "free_x", space = "free_x"
  ) +
  scale_x_discrete(position = "top") +
  scale_fill_viridis_c(
    name = NULL,
    labels = c("lowest", "highest (GB/s)"), breaks = c(0, 1)
  ) +
  scale_color_manual(values = c(`TRUE` = "grey10", `FALSE` = "white"),
                     guide = "none") +
  labs(
    title = "Mean throughput (GB/s) across hash functions",
    caption = paste(
      "Shaded relative to the fastest hash for each payload and output size",
      "(e.g. fx has the highest throughput for a 64bit value at a 1 byte",
      "payload, so it is brightest)",
      sep = "\n"
    ),
    x = "Hash Library",
    y = "Payload Size (bytes)"
  ) +
  theme(
    axis.text.x.top = element_text(angle = 45, hjust = 0, vjust = 0),
    legend.position = "bottom",
    panel.grid = element_blank(),
    panel.spacing.x = unit(8, "pt"),
    strip.background = element_rect(fill = "grey90", color = NA),
    strip.text = element_text(face = "bold", margin = margin(4, 4, 4, 4))
  )
ggsave("highwayhash-table.png", width = 8, height = 6, dpi = 100)
