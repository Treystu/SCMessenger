#!/usr/bin/env python3
"""Acceptance tests for the offline delivery-probability evaluator (#666).

The issue's acceptance rows, as tests:

* exactness <= 1e-9 against brute force on <= 20-link networks;
* Monte Carlo agrees with the exact value within 3 sigma at a stated sample count;
* rare-event mode reaches <= 10% relative error at a failure probability of 1e-4;
* every result states the regime it came from.
"""

import random
import unittest

import scripts.delivery_probability as dp

TOL = 1e-9


def topology(links, source="a", sink="b"):
    nodes = sorted({a for a, _, _ in links} | {b for _, b, _ in links} | {source, sink})
    return {"source": source, "sink": sink, "nodes": nodes,
            "links": [(a, b, float(p)) for a, b, p in links]}


class ExactTests(unittest.TestCase):
    def test_series_links_multiply(self):
        res = dp.exact_reliability(topology([("a", "m", 0.9), ("m", "b", 0.9)]))
        self.assertAlmostEqual(res["delivery_probability"], 0.81, delta=TOL)
        self.assertTrue(res["bruteforce_checked"])

    def test_parallel_links_combine(self):
        res = dp.exact_reliability(topology([("a", "b", 0.9), ("a", "b", 0.9)]))
        self.assertAlmostEqual(res["delivery_probability"], 0.99, delta=TOL)

    def test_bridge_is_neither_series_nor_parallel(self):
        # a-m1, a-m2, m1-b, m2-b: delivery unless a whole side fails.
        links = [("a", "m1", 0.9), ("a", "m2", 0.9), ("m1", "b", 0.9), ("m2", "b", 0.9)]
        res = dp.exact_reliability(topology(links))
        expected = dp.exact_bruteforce(topology(links))
        self.assertAlmostEqual(res["delivery_probability"], expected, delta=TOL)

    def test_decomposition_matches_bruteforce_on_random_networks(self):
        rng = random.Random(7)
        for _ in range(12):
            n_links = rng.randint(2, 8)
            n_nodes = rng.randint(2, 4)
            names = [f"n{i}" for i in range(n_nodes)]
            links = []
            for _ in range(n_links):
                a, b = rng.sample(names, 2)
                links.append((a, b, round(rng.uniform(0.5, 0.99), 4)))
            topo = topology(links, source=names[0], sink=names[-1])
            res = dp.exact_reliability(topo)
            self.assertAlmostEqual(res["delivery_probability"], dp.exact_bruteforce(topo), delta=TOL)
            self.assertLessEqual(res["bruteforce_delta"], TOL)

    def test_disconnected_topology_is_zero(self):
        links = [("a", "m", 0.9), ("x", "b", 0.9)]
        res = dp.exact_reliability(topology(links))
        self.assertEqual(res["delivery_probability"], 0.0)

    def test_unsampled_bruteforce_is_reported_not_implied(self):
        names = [f"n{i}" for i in range(6)]
        links = []
        for i in range(21):
            a, b = names[i % 5], names[(i % 5) + 1]
            links.append((a, b, 0.97))
        res = dp.exact_reliability(topology(links, source="n0", sink="n5"))
        self.assertFalse(res["bruteforce_checked"])
        self.assertIsNone(res["bruteforce_delta"])


class MonteCarloTests(unittest.TestCase):
    def test_mc_agrees_with_exact_within_three_sigma(self):
        links = [("a", "m1", 0.9), ("a", "m2", 0.9), ("m1", "b", 0.9), ("m2", "b", 0.9)]
        topo = topology(links)
        exact = dp.exact_reliability(topo)["delivery_probability"]
        for seed in (1, 2, 3):
            res = dp.monte_carlo(topo, samples=20_000, seed=seed)
            sigma = res["standard_error"] or 1.0 / (20_000 ** 0.5)
            self.assertLessEqual(abs(res["delivery_probability"] - exact), 3 * max(sigma, 1e-9),
                                 f"seed {seed}: {res}")
        self.assertEqual(res["mode"], "mc")

    def test_all_success_run_reports_resolution_not_zero(self):
        topo = topology([("a", "b", 0.999999)])
        res = dp.monte_carlo(topo, samples=1_000, seed=5)
        self.assertEqual(res["failures_observed"], 0)
        self.assertIn("resolution", res["notes"])


class RareEventTests(unittest.TestCase):
    def test_ten_percent_relative_error_at_one_e_minus_four(self):
        # One link with failure probability exactly 1e-4: plain MC cannot resolve
        # this at a sane sample count, importance sampling can.
        topo = topology([("a", "b", 0.9999)])
        res = dp.rare_event_failure_probability(topo, samples=40_000, seed=11)
        self.assertLessEqual(res["relative_error"], 0.10, res)
        self.assertTrue(res["relative_error_within_10pct"])
        self.assertAlmostEqual(res["failure_probability"], 1e-4, delta=3e-5)

    def test_bias_is_chosen_to_make_the_event_measurable(self):
        topo = topology([("a", "b", 0.9999)])
        res = dp.rare_event_failure_probability(topo, samples=5_000, seed=3)
        self.assertGreater(res["failure_events_sampled"], 0)
        self.assertGreater(res["failure_inflation"], 1.0)

    def test_mesh_failure_matches_exact(self):
        links = [("a", "m", 0.9), ("m", "b", 0.9), ("a", "b", 0.5)]
        topo = topology(links)
        exact_q = 1.0 - dp.exact_reliability(topo)["delivery_probability"]
        res = dp.rare_event_failure_probability(topo, samples=20_000, seed=2)
        self.assertLessEqual(abs(res["failure_probability"] - exact_q), 0.02, (res, exact_q))


class RegimeReportingTests(unittest.TestCase):
    def test_every_mode_states_itself(self):
        topo = topology([("a", "b", 0.9)])
        self.assertEqual(dp.exact_reliability(topo)["mode"], "exact")
        self.assertEqual(dp.monte_carlo(topo, samples=500, seed=0)["mode"], "mc")
        self.assertEqual(dp.rare_event_failure_probability(topo, samples=2_000, seed=0)["mode"], "rare")


if __name__ == "__main__":
    unittest.main()
