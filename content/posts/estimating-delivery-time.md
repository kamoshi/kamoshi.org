---
title: "Estimating package delivery time"
date: 2022-06-21T18:44:53+02:00
tags: [math, statistics]
---

In the last few months I have ordered a dozen of items from Japan. In total there were six distinct orders that were delivered to me in separate boxes. Here's the data for the time between being sent out (dispatch from outward office of exchange) from Japan and arriving (arrival at inward office of exchange) in Poland.

| Location      | Outbound        | Inbound         | Diff days   |
|---------------|-----------------|-----------------|-------------|
| TOKYO INT BAG | 3/29/2022 4:20  | 4/3/2022 6:22   | 5.084722222 |
| OSAKA INT BAG | 4/9/2022 14:30  | 4/15/2022 6:38  | 5.672222222 |
| OSAKA INT BAG | 4/11/2022 14:30 | 4/22/2022 5:29  | 10.62430556 |
| TOKYO INT BAG | 4/19/2022 13:20 | 4/22/2022 6:05  | 2.697916667 |
| OSAKA INT BAG | 4/21/2022 14:30 | 4/28/2022 18:33 | 7.16875     |
| TOKYO INT BAG | 5/10/2022 13:20 | 5/18/2022 2:37  | 7.553472222 |

We have $n = 6$ samples and using this data we can calculate the mean $\bar X$ as well as the standard deviation $s$ of the sample.

$$
\bar X \approx 6.466898148
$$
$$
s \approx 2.672243145
$$

We do not know the properties of the population, and we only have 6 samples, so we have to use the Student's t-distribution here. There are $k = n - 1 = 5$ degrees of freedom.

Let's assume confidence level of 95%.
$$
\alpha = 1 - 0.95 = 0.05
$$

We can use the table for t-distribution to find that the value of $t_{\alpha, n-1} = 2.571$. Now we can calculate the confidence interval for the mean $\mu$ delivery time between Japan and Poland.

$$
(\bar X - t_{\alpha, n-1} \frac{s}{\sqrt{n}}; \bar X + t_{\alpha, n-1} \frac{s}{\sqrt{n}})
$$
$$
t_{\alpha, n-1} \frac{s}{\sqrt{n}} \approx 2.804347194
$$
$$
(\bar X - 2.804347194; \bar X + 2.804347194) = (3.662550954; 9.271245342)
$$

With these results we can say with 95% confidence that the mean time of delivery from Japan to Poland lies somewhere between 3.66 days and 9.27 days. We can also calculate different results for different confidence levels to get some more interesting overview:

| Confidence level | $t_{\alpha, n-1} \frac{s}{\sqrt{n}}$ | Confidence interval        |
|------------------|------------------------------------------|----------------------------|
| 99%              | 4.398820806                              | (2.068077342; 10.86571895) |
| 95%              | 2.804347194                              | (3.662550954; 9.271245342) |
| 90%              | 2.198294244                              | (4.268603904; 8.665192392) |
| 80%              | 1.610099019                              | (4.856799129; 8.076997167) |
| 50%              | 0.792770797                              | (5.674127351; 7.259668946) |
