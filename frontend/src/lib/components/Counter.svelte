<script lang="ts">
	import { untrack } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { countUp } from '#lib/fx/celebrate.ts';

	/**
	 * A number that counts to its new value instead of jumping. `from` sets
	 * where the first count starts (a prize counting up from 0); later changes
	 * count from what is on screen.
	 */
	interface Props {
		value: number;
		from?: number;
		/** ms */
		duration?: number;
		testid?: string;
	}
	let { value, from, duration = 700, testid }: Props = $props();
	const i18n = getI18n();

	// svelte-ignore state_referenced_locally
	let shown = $state(from ?? value);
	let bumping = $state(false);

	$effect(() => {
		const target = value;
		// Where the count starts, not a dependency: it changes every frame
		const start = untrack(() => shown);
		if (start === target) return;
		const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reduced) {
			shown = target;
			return;
		}
		bumping = target > start;
		const t0 = performance.now();
		let frame = requestAnimationFrame(function tick(t) {
			shown = countUp(start, target, t - t0, duration);
			if (shown !== target) frame = requestAnimationFrame(tick);
			else bumping = false;
		});
		return () => cancelAnimationFrame(frame);
	});
</script>

<!-- The final value for assistive tech and tests; the counting is for the eyes -->
<span class="counter num" class:bumping data-testid={testid} data-value={value}
	>{i18n.num(shown)}</span
>

<style>
	.counter {
		display: inline-block;
		transition: transform 0.2s ease;
	}
	.bumping {
		transform: scale(1.12);
		color: var(--gold);
	}
</style>
