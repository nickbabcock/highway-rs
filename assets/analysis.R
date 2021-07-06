library(scales)
library(tidyverse)
library(readr)
library(ggnewscale)

is_highwayhash <- Vectorize(function(fn) {
  switch(fn,
         "avx" = TRUE,
         "sse" = TRUE,
         "portable" = TRUE,
         FALSE)
})

get_line_type <- Vectorize(function(fn) {
  switch(fn,
         "avx" = "highwayhash",
         "sse" = "highwayhash",
         "portable" = "highwayhash",
         "other")
})

df <- read_csv("./highway.csv")
df <- mutate(
  df,
  fn = `function`,
  highwayhash = is_highwayhash(fn),
  line = get_line_type(fn),
  throughput = value * iteration_count * 10 ^ 9 / sample_measured_value,
  hashes_per_ms = iteration_count * 10 ^ 6 / sample_measured_value,
)

df64 <- df %>% filter(group == '64bit')
df64highway <- df64 %>% filter(highwayhash == TRUE)
df64other <- df64 %>% filter(highwayhash == FALSE)

df256 <- df %>% filter(group == '256bit')
df256highway <- df256 %>% filter(highwayhash == TRUE)
df256other <- df256 %>% filter(highwayhash == FALSE)

# We create a custom palette as we want hashes that produce both 64bit and
# 256bit results to have consistent colors between multiple graphs. Hashes
# that don't produce both are fine to have inconsistent colors so that we can
# use a smaller color palette (and a smaller color palette makes it easier to
# read the graph)
df64UniqueNames <- df64 %>% select(fn) %>% distinct() %>% pull() %>% sort()
df256UniqueNames <- df256 %>% select(fn) %>% distinct() %>% pull() %>% sort()
namesInBoth <- intersect(df64UniqueNames, df256UniqueNames)
neededColors <- max(length(df64UniqueNames), length(df256UniqueNames))
pal64 <- brewer.pal(neededColors, "Set1")[1:length(df64UniqueNames)]
pal256 <- brewer.pal(neededColors, "Set1")[1:length(df256UniqueNames)]
names(pal64) <- c(namesInBoth, setdiff(df64UniqueNames, namesInBoth))
names(pal256) <- c(namesInBoth, setdiff(df256UniqueNames, namesInBoth))

byte_rate <- function(l) {
  paste(scales::number_bytes(l, symbol = "GB", units = "si"), "/s")
}

ggplot(mapping=aes(value, throughput)) +
  stat_summary(data=df64, mapping=aes(value, throughput, color = fn), fun = mean, geom="point", size = 1.5) +
  scale_color_manual("Points", values=pal64, guide=FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df64highway, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("HighwayHash", values=pal64, guide=guide_legend(order = 1)) +
  scale_linetype(guide = FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df64other, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("Other Hashes", values=pal64, guide=guide_legend(order = 2)) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 64bit hash functions at varying payload lengths",
       caption = "solid lines are HighwayHash functions",
       col = "Hash function",
       y = "Throughput", 
       x = "Payload length in bytes")
ggsave('64bit-highwayhash.png', width = 8, height = 5, dpi = 100)

ggplot(mapping=aes(value, throughput)) +
  stat_summary(data=df256, mapping=aes(value, throughput, color = fn), fun = mean, geom="point", size = 1.5) +
  scale_color_manual("Points", values=pal256, guide=FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df256highway, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("HighwayHash", values=pal256, guide=guide_legend(order = 1)) +
  scale_linetype(guide = FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df256other, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("Other Hashes", values=pal256, guide=guide_legend(order = 2)) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 256bit hash functions at varying payload lengths",
       caption = "solid lines are HighwayHash functions",
       col = "Hash function",
       y = "Throughput",
       x = "Payload length in bytes")

ggsave('256bit-highwayhash.png', width = 8, height = 5, dpi = 100)

ggplot(mapping=aes(value, hashes_per_ms)) +
  stat_summary(data=df256, mapping=aes(value, hashes_per_ms, color = fn), fun = mean, geom="point", size = 1.5) +
  scale_color_manual("Points", values=pal256, guide=FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df256highway, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("HighwayHash", values=pal256, guide=guide_legend(order = 1)) +
  scale_linetype(guide = FALSE) +
  ggnewscale::new_scale_color() +
  stat_summary(data=df256other, mapping=aes(linetype = line, color = fn), fun = mean, geom="line", size = 1.2) +
  scale_color_manual("Other Hashes", values=pal256, guide=guide_legend(order = 2)) +
  scale_y_continuous(limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of hash rate for 256bit hash functions at varying payload lengths",
       caption = "solid lines are HighwayHash functions",
       col = "Hash function",
       y = "Hashes per ms",
       x = "Payload length in bytes")
ggsave('256bit-highwayhash-rate.png', width = 8, height = 5, dpi = 100)

ggplot(df %>% filter(highwayhash == TRUE), aes(value, throughput, color = fn, line_type = group)) + 
  stat_summary(fun = mean, geom="point", size = 1.5) +
  stat_summary(aes(linetype = as.factor(group)), fun = mean, geom="line", size = 1.2) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 64bit vs 256bit HighwayHash",
       col = "HighwayHash",
       linetype = "Output",
       y = "Throughput",
       x = "Payload length in bytes") +
  scale_colour_manual(values = pal256)
ggsave('64bit-vs-256bit-highwayhash.png', width = 8, height = 5, dpi = 100)

reldf <- df %>%
  mutate(throughput = throughput / 10^9) %>%
  group_by(group, fn, highwayhash, value) %>%
  summarize(throughput = mean(throughput)) %>%
  ungroup() %>%
  group_by(value, group) %>%
  mutate(relative = throughput / max(throughput)) %>%
  ungroup() %>%
  complete(group, fn, value, fill = list(highwayhash = FALSE))

ordered <- reldf %>% distinct(fn, highwayhash) %>% arrange(!highwayhash, fn) %>% pull(fn)

# Group all highway hash functions next to each other in the graph
reldf$fn <- factor(reldf$fn, levels = ordered)

ggplot(reldf, aes(fn, as.factor(value))) +
  geom_tile(aes(fill = relative), color = "white") +
  facet_grid(group ~ .) +
  scale_x_discrete(position = "top") +
  scale_fill_gradient(name = "", low = "white", high = "steelblue", na.value = "#D8D8D8", labels = c("lowest", "highest (GB/s)"), breaks = c(0,1)) +
  xlab("Hash Library") +
  ylab("Payload Size (bytes)") +
  geom_text(size = 3.25, aes(label = ifelse(is.na(relative), "NA", format(round(throughput, 2), digits = 3)))) +
  theme(axis.text.x.top=element_text(angle=45, hjust=0, vjust=0)) +
  theme(legend.position="bottom") +
  theme(plot.caption = element_text(hjust=0)) +
  ggtitle("Comparison of Mean Throughput (GB/s) across Hash Functions") +
  labs(caption = "Shaded relative by payload and return size\n(eg: fnv has the highest throughput for a 64bit value with a 1 byte payload, so it is a deep blue)")
ggsave('highwayhash-table.png', width = 8, height = 6, dpi = 100)
