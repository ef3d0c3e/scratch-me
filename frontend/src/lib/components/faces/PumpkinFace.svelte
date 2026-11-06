<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { PumpkinLayout } from '#lib/types.ts';

	interface Props {
		layout: PumpkinLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
</script>

<p class="rules">{i18n.t('series.pumpkin_patch.rules')}</p>
<div class="play square">
	<div class="patch">
		{#each { length: 9 }, i (i)}
			{@const p = layout?.pumpkins[i]}
			<div class="pumpkin" class:golden={p?.kind === 'golden'} data-pumpkin={p?.kind ?? ''}>
				{#if p}
					<span class="face">{p.kind === 'ghost' ? '👻' : p.kind === 'golden' ? '🌟' : '🎃'}</span>
					{#if p.kind === 'candy'}<b>{'🍬'.repeat(p.candy)}</b>{/if}
				{/if}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
{#if revealed && layout}
	<p class="total" data-testid="candy">{i18n.t('pumpkin.total', { n: layout.candy })}</p>
{/if}
<p class="pays">{i18n.t('pumpkin.pays')}</p>

<style>
	.patch {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		grid-template-rows: repeat(3, 1fr);
		gap: 8px;
		padding: 10px;
		border-radius: 14px;
		background: radial-gradient(circle, #3f2a14, #1c1208);
	}
	.pumpkin {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border-radius: 12px;
		background: rgba(249, 115, 22, 0.15);
	}
	.pumpkin.golden {
		background: radial-gradient(circle, #fde68a, #f59e0b);
	}
	.face {
		font-size: 40px;
	}
	b {
		font-size: 14px;
		letter-spacing: -2px;
	}
	.total {
		text-align: center;
		font: 400 20px/1 var(--display);
		color: #9a3412;
		margin-top: 10px;
	}
	.pays {
		text-align: center;
		font-size: 12px;
		font-weight: 700;
		margin-top: 6px;
	}
</style>
