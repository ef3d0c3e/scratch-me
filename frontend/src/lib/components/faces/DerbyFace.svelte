<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { DerbyLayout } from '#lib/types.ts';

	/** Your two horses under the foil; once they show, the race runs */
	interface Props {
		layout: DerbyLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	const COLORS = ['#e0243c', '#3651f5', '#16a34a', '#ffc23d', '#a21caf', '#0ea5e9'];
	const RACE = 4000;

	let running = $state(false);
	let finished = $state(false);
	let started = false;
	$effect(() => {
		if (!revealed || !layout || started) return;
		started = true;
		const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
		// A frame with the horses at the start, then off they go
		requestAnimationFrame(() => (running = true));
		setTimeout(
			() => {
				finished = true;
				onsettled();
			},
			reduced ? 100 : RACE + 300
		);
	});
	/** Finishing place, from 0: the winner gets there first */
	const place = (horse: number) => layout?.order.indexOf(horse) ?? 0;
	const mine = (horse: number) => !!layout?.bets.some((b) => b.horse === horse);
</script>

<p class="rules">{i18n.t('series.derby.rules')}</p>
<p class="label">{i18n.t('derby.yours')}</p>
<div class="play slips">
	<div class="bets">
		{#each { length: 2 }, i (i)}
			{@const b = layout?.bets[i]}
			<div class="slip" style:--c={b ? COLORS[b.horse - 1] : 'transparent'}>
				{#if b}
					<span class="num">{b.horse}</span>
					<b>{i18n.num(b.coins)}</b>
				{/if}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
<div class="track" aria-hidden="true">
	{#each { length: 6 }, h (h)}
		{@const horse = h + 1}
		<div class="lane" class:mine={revealed && mine(horse)}>
			<span class="gate">{horse}</span>
			<span
				class="horse"
				class:running
				style:--c={COLORS[h]}
				style:--time="{RACE * (0.82 + place(horse) * 0.035)}ms"
				data-horse={horse}>🏇</span
			>
		</div>
	{/each}
	<span class="finish"></span>
</div>
{#if finished && layout}
	<p class="winner" data-testid="derby-winner">{i18n.t('derby.winner', { n: layout.order[0] })}</p>
{/if}

<style>
	.label {
		font-size: 11px;
		font-weight: 800;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		text-align: center;
		opacity: 0.7;
		margin-bottom: 6px;
	}
	.slips {
		aspect-ratio: 2 / 0.7 !important;
	}
	.bets {
		height: 100%;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
		padding: 8px;
	}
	.slip {
		display: flex;
		align-items: center;
		justify-content: space-around;
		border-radius: 12px;
		background: var(--paper);
		box-shadow: inset 6px 0 0 var(--c);
	}
	.slip .num {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		background: var(--c);
		color: #fff;
		font: 400 20px/1 var(--display);
	}
	.slip b {
		font: 400 20px/1 var(--display);
		color: #166534;
	}
	.track {
		position: relative;
		margin-top: 12px;
		padding: 6px 10px;
		border-radius: 12px;
		background: repeating-linear-gradient(#5aa04a 0 22px, #4e9140 22px 44px);
	}
	.lane {
		position: relative;
		height: 22px;
		display: flex;
		align-items: center;
	}
	.lane.mine {
		background: rgba(255, 255, 255, 0.25);
		border-radius: 6px;
	}
	.gate {
		width: 18px;
		font: 700 11px/1 var(--body);
		color: #fff;
	}
	.horse {
		position: absolute;
		left: 20px;
		font-size: 18px;
		/* Facing the finish line */
		transform: scaleX(-1);
		filter: drop-shadow(0 0 0 var(--c)) drop-shadow(1px 1px 0 var(--c));
	}
	.horse.running {
		left: calc(100% - 34px);
		transition: left var(--time) cubic-bezier(0.45, 0.05, 0.4, 1);
		animation: gallop 0.25s ease-in-out infinite alternate;
	}
	@keyframes gallop {
		to {
			translate: 0 -3px;
		}
	}
	.finish {
		position: absolute;
		top: 4px;
		bottom: 4px;
		right: 28px;
		width: 4px;
		background: repeating-linear-gradient(#fff 0 4px, #1b1530 4px 8px);
	}
	.winner {
		text-align: center;
		font: 400 18px/1 var(--display);
		margin-top: 10px;
		color: #166534;
	}
</style>
