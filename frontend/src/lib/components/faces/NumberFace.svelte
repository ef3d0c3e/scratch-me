<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { NumberLayout } from '#lib/types.ts';

	interface Props {
		layout: NumberLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();

	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const matched = (n: number) => revealed && !!layout?.spots.some((s) => s.hit && s.n === n);
</script>

<p class="rules">{i18n.t('series.number_match.rules')}</p>
<div class="play">
	<div class="board">
		<p class="label">{i18n.t('number.winning')}</p>
		<div class="winning">
			{#each { length: 3 }, i (i)}
				{@const n = layout?.winning[i]}
				<span class="ball" class:lit={n !== undefined && matched(n)}>{n ?? ''}</span>
			{/each}
		</div>
		<p class="label">{i18n.t('number.yours')}</p>
		<div class="spots">
			{#each { length: 8 }, i (i)}
				{@const s = layout?.spots[i]}
				<span
					class="spot"
					class:hit={revealed && s?.hit}
					data-hit={revealed && s?.hit ? '' : undefined}
				>
					<b>{s?.n ?? ''}</b>
					<small>{s ? i18n.num(s.coins) : ''}</small>
				</span>
			{/each}
		</div>
	</div>
	{@render foil()}
</div>

<style>
	.board {
		display: grid;
		gap: 6px;
		padding: 12px;
		height: 100%;
		grid-template-rows: auto 1fr auto 2fr;
	}
	.label {
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		opacity: 0.7;
		text-align: center;
	}
	.winning {
		display: flex;
		justify-content: center;
		gap: 14px;
	}
	.ball {
		width: 58px;
		height: 58px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		font: 400 24px/1 var(--display);
		color: #fff;
		background: radial-gradient(circle at 35% 30%, #9fdcff, #57b8ff 45%, #1f6fc2);
		box-shadow: inset 0 -4px 0 rgba(0, 0, 0, 0.2);
	}
	.ball.lit {
		animation: glow 1s ease-in-out infinite alternate;
	}
	.spots {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
	}
	.spot {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border-radius: 12px;
		background: var(--paper);
		border: 2px dashed #e9cf9a;
	}
	.spot b {
		font: 400 22px/1 var(--display);
	}
	.spot small {
		font-size: 12px;
		font-weight: 800;
		color: #1f6fc2;
	}
	.spot.hit {
		background: radial-gradient(circle, #fff7d1, #ffd66b);
		border: 2px solid var(--gold-deep);
		animation: pop 0.8s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes pop {
		40% {
			transform: scale(1.15) rotate(-3deg);
		}
	}
	@keyframes glow {
		to {
			box-shadow:
				inset 0 -4px 0 rgba(0, 0, 0, 0.2),
				0 0 0 5px rgba(255, 194, 61, 0.7);
		}
	}
</style>
