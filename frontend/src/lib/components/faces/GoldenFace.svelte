<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { SERIES_LOOK, cosmeticKey } from '#lib/series.ts';
	import type { GoldenLayout } from '#lib/types.ts';
	import CosmeticBadge from '../CosmeticBadge.svelte';

	interface Props {
		layout: GoldenLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();

	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const p = $derived(layout?.prize);
</script>

<p class="rules">{i18n.t('series.golden.rules')}</p>
<div class="play golden-play">
	<div class="prize" class:revealed>
		{#if p?.kind === 'cosmetic'}
			<CosmeticBadge item={p.item} size="big" />
			<span class="name">{i18n.t(cosmeticKey(p.item))}</span>
		{:else if p?.kind === 'coins'}
			<span class="coins">{i18n.num(p.amount)}</span>
			<span class="name">{i18n.t('wheel.coins')}</span>
		{:else if p?.kind === 'tickets'}
			<div class="bundle">
				{#each p.series as s, i (i)}
					<span
						class="stub"
						style:--edge={SERIES_LOOK[s].edge}
						style:--edge-ink={SERIES_LOOK[s].ink}>{i18n.t(`series.${s}`)}</span
					>
				{/each}
			</div>
		{/if}
	</div>
	{@render foil()}
</div>

<style>
	.prize {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		border-radius: 14px;
		background:
			repeating-conic-gradient(
				from 0deg,
				rgba(255, 194, 61, 0.18) 0deg 12deg,
				transparent 12deg 24deg
			),
			radial-gradient(circle, #fff7d1, #ffe7a3);
	}
	.prize.revealed {
		animation: glow 1.4s ease-in-out infinite alternate;
	}
	@keyframes glow {
		to {
			box-shadow: inset 0 0 40px rgba(255, 194, 61, 0.8);
		}
	}
	.name {
		font: 400 22px/1 var(--display);
		color: #6b3d00;
	}
	.coins {
		font: 400 56px/1 var(--display);
		color: #c77d00;
		text-shadow: 0 3px 0 #6b3d00;
	}
	.bundle {
		display: grid;
		gap: 8px;
		width: 70%;
	}
	.stub {
		padding: 8px;
		border-radius: 8px;
		text-align: center;
		font: 400 14px/1 var(--display);
		color: var(--edge-ink);
		background: var(--paper-2);
		box-shadow: 0 0 0 2.5px var(--edge);
	}
</style>
