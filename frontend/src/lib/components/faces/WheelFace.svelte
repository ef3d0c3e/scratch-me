<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { WheelLayout } from '#lib/types.ts';
	import { wheelTurn } from '#lib/fx/wheel.ts';

	/** The spot is scratched, then the wheel spins: the prize shows once it stops */
	interface Props {
		layout: WheelLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		/** A tick for each segment passing the pointer */
		ontick?: () => void;
	}
	let { layout, revealed, foil, onsettled, ontick }: Props = $props();
	const i18n = getI18n();

	const SEGMENTS = 12;
	const shown = $derived(layout?.wheel ?? new Array(SEGMENTS).fill(1));
	let turn = $state(0);
	let spinning = $state(false);
	let stopped = $state(false);
	/** Not state: the effect must not run again when the spin starts */
	let started = false;
	let wheel = $state<HTMLElement>();
	const COLORS = ['#34e0a1', '#fff4dc', '#57b8ff', '#fff4dc', '#ff4f8b', '#fff4dc'];

	$effect(() => {
		if (!revealed || !layout || started) return;
		started = true;
		// An empty spot has nothing to multiply
		if (layout.base === 0) {
			onsettled();
			return;
		}
		spinning = true;
		turn = wheelTurn(layout.stop, layout.wheel.length);
		const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const ms = reduced ? 50 : 4200;
		// Ticks slow down with the wheel
		if (ontick && !reduced) {
			let t = 0;
			const tick = () => {
				ontick();
				t += 1;
				const gap = 40 + t * t * 0.9;
				if (gap < 600) setTimeout(tick, gap);
			};
			tick();
		}
		setTimeout(() => {
			stopped = true;
			onsettled();
		}, ms + 150);
	});
</script>

<p class="rules">{i18n.t('series.cash_wheel.rules')}</p>
<div class="spot-row">
	<div class="play spot">
		<div class="base" data-base={layout?.base}>
			{#if layout}
				{#if layout.base > 0}
					<span class="amount">{i18n.num(layout.base)}</span>
					<span class="unit">{i18n.t('wheel.coins')}</span>
				{:else}
					<span class="empty">{i18n.t('wheel.empty')}</span>
				{/if}
			{/if}
		</div>
		{@render foil()}
	</div>
</div>
<div class="wheel-wrap" aria-hidden="true">
	<span class="pointer"></span>
	<div
		class="wheel"
		bind:this={wheel}
		class:spinning
		style:transform="rotate({turn}deg)"
		style:--n={SEGMENTS}
	>
		<svg viewBox="-100 -100 200 200">
			{#each shown as m, i (i)}
				{@const a0 = ((i * 360) / SEGMENTS - 90) * (Math.PI / 180)}
				{@const a1 = (((i + 1) * 360) / SEGMENTS - 90) * (Math.PI / 180)}
				{@const mid = ((i + 0.5) * 360) / SEGMENTS}
				<path
					d="M0 0 L{Math.cos(a0) * 96} {Math.sin(a0) * 96} A96 96 0 0 1 {Math.cos(a1) *
						96} {Math.sin(a1) * 96} Z"
					fill={m >= 10 ? '#ffc23d' : COLORS[i % COLORS.length]}
					stroke="#2a1650"
					stroke-width="1.5"
				/>
				<text
					transform="rotate({mid}) translate(0 -70)"
					text-anchor="middle"
					dominant-baseline="middle"
					class:big={m >= 10}>×{m}</text
				>
			{/each}
			<circle r="16" fill="#2a1650" />
			<circle r="10" fill="#ffc23d" />
		</svg>
	</div>
</div>
{#if stopped && layout}
	<p class="mult" data-testid="wheel-result">
		{i18n.t('wheel.result', { base: i18n.num(layout.base), mult: layout.wheel[layout.stop] })}
	</p>
{/if}

<style>
	.spot-row {
		display: flex;
		justify-content: center;
	}
	.spot {
		width: 70%;
	}
	.base {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: 14px;
		background: var(--paper);
		border: 2px dashed #b7e8d4;
	}
	.amount {
		font: 400 34px/1 var(--display);
		color: #119469;
	}
	.unit {
		font-weight: 800;
	}
	.empty {
		font: 400 20px/1 var(--display);
		opacity: 0.5;
	}
	.wheel-wrap {
		position: relative;
		width: min(250px, 66vw);
		aspect-ratio: 1;
		margin: 14px auto 0;
	}
	.pointer {
		position: absolute;
		left: 50%;
		top: -6px;
		z-index: 1;
		width: 0;
		height: 0;
		transform: translateX(-50%);
		border-left: 12px solid transparent;
		border-right: 12px solid transparent;
		border-top: 22px solid #2a1650;
		filter: drop-shadow(0 2px 0 #ffc23d);
	}
	.wheel {
		width: 100%;
		height: 100%;
		border-radius: 50%;
		box-shadow:
			0 0 0 5px #2a1650,
			0 0 0 9px #34e0a1;
	}
	.wheel.spinning {
		transition: transform 4.2s cubic-bezier(0.12, 0.7, 0.15, 1);
	}
	svg {
		width: 100%;
		height: 100%;
		display: block;
	}
	text {
		font: 400 15px var(--display);
		fill: #2a1650;
	}
	text.big {
		font-size: 17px;
		fill: #6b3d00;
	}
	.mult {
		text-align: center;
		font-weight: 800;
		margin-top: 8px;
		color: #119469;
	}
</style>
