library(scales)
library(tidyverse)
library(readr)

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
df <- mutate(df,
             fn = `function`,
             highwayhash = is_highwayhash(fn),
             line = get_line_type(fn),
             throughput = value * iteration_count * 10^9 / sample_time_nanos)


pal <- hue_pal()(df %>% select(fn) %>% distinct() %>% count() %>% pull())
names(pal) <- df %>% select(fn) %>% distinct() %>% pull()

df64 <- df %>% filter(group == '64bit')
df256 <- df %>% filter(group == '256bit')

byte_rate <- function(l) {
  paste(scales::number_bytes(l, symbol = "GB", units = "si"), "/s")
}

ggplot(df64, aes(value, throughput, color = fn)) + 
  stat_summary(fun.y = mean, geom="point", size = 1.5) +
  stat_summary(aes(linetype = line), fun.y = mean, geom="line", size = 1.2) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 64bit hash functions at varying payload lengths",
       caption = "solid lines are highway hash functions",
       col = "Hash function",
       y = "Throughput",
       x = "Payload length in bytes") +
  guides(linetype = FALSE) +
  scale_colour_manual( values = pal)
ggsave('64bit-highwayhash.png', width = 8, height = 5, dpi = 100)

ggplot(df256, aes(value, throughput, color = fn)) + 
  stat_summary(fun.y = mean, geom="point", size = 1.5) +
  stat_summary(aes(linetype = line), fun.y = mean, geom="line", size = 1.2) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 256bit hash functions at varying payload lengths",
       caption = "solid lines are highway hash functions",
       col = "Hash function",
       y = "Throughput",
       x = "Payload length in bytes") +
  guides(linetype = FALSE) +
  scale_colour_manual(values = pal)
ggsave('256bit-highwayhash.png', width = 8, height = 5, dpi = 100)

ggplot(df %>% filter(highwayhash == TRUE), aes(value, throughput, color = fn, line_type = group)) + 
  stat_summary(fun.y = mean, geom="point", size = 1.5) +
  stat_summary(aes(linetype = as.factor(group)), fun.y = mean, geom="line", size = 1.2) +
  scale_y_continuous(labels = byte_rate, limits = c(0, NA), breaks = pretty_breaks(10)) +
  scale_x_continuous(trans='log2', limit = c(1, NA), breaks = c(1, 4, 16, 64, 256, 1024, 4096, 16384, 65536)) +
  labs(title = "Comparison of throughput for 64bit vs 256bit highway hash",
       caption = "solid lines are 256bit",
       col = "Hash function",
       y = "Throughput",
       x = "Payload length in bytes")  +
  guides(linetype = FALSE) +
  scale_colour_manual(values = pal)
ggsave('64bit-vs-256it-highwayhash.png', width = 8, height = 5, dpi = 100)
