<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { FishingLayout } from '#lib/types.ts';

	interface Props {
		layout: FishingLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	/** What the scratch put in the aquarium: the server says, as it decided it */
	type Catch = { new: string[]; caught: number; of: number };
	const LOOK: Record<string, string> = {
		sardine: '🐟',
		perch: '🐠',
		mackerel: '🐟',
		carp: '🐟',
		trout: '🐠',
		pike: '🐠',
		salmon: '🐠',
		tuna: '🐟',
		swordfish: '🐬',
		pufferfish: '🐡',
		octopus: '🐙',
		golden_koi: '🎏'
	};
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const caught = $derived((layout as { aquarium?: Catch } | null)?.aquarium ?? null);
	const isNew = (f: string) => !!caught?.new.includes(f);
</script>

<p class="rules">{i18n.t('series.fishing.rules')}</p>
<div class="play pond">
	<div class="casts">
		{#each { length: 4 }, i (i)}
			{@const c = layout?.casts[i]}
			<div class="cast" data-fish={c ? (c.fish ?? 'boot') : ''}>
				{#if c}
					<span class="fish">{c.fish ? LOOK[c.fish] : '🥾'}</span>
					<small>{c.fish ? i18n.t(`fish.${c.fish}` as never) : i18n.t('fish.boot')}</small>
					<b>{c.coins ? i18n.num(c.coins) : ''}</b>
					{#if revealed && c.fish && isNew(c.fish)}<span class="new">{i18n.t('fish.new')}</span
						>{/if}
				{/if}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>
{#if revealed && caught}
	<p class="aquarium" data-testid="aquarium">
		🐠 {i18n.t('fish.aquarium', { n: caught.caught, of: caught.of })}
	</p>
{/if}

<style>
	.pond {
		aspect-ratio: 4 / 1.6 !important;
	}
	.casts {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 6px;
		padding: 8px;
		border-radius: 14px;
		background: linear-gradient(#67e8f9, #0891b2);
	}
	.cast {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.35);
	}
	.fish {
		font-size: 34px;
	}
	small {
		font-size: 11px;
		font-weight: 700;
		color: #083344;
		text-align: center;
	}
	b {
		font-size: 13px;
		color: #0e7490;
	}
	.new {
		position: absolute;
		top: 4px;
		right: 4px;
		font: 400 9px/1 var(--display);
		padding: 2px 4px;
		border-radius: 4px;
		background: #fde047;
		color: #713f12;
	}
	.aquarium {
		text-align: center;
		font-weight: 800;
		margin-top: 10px;
		color: #0e7490;
	}
</style>
