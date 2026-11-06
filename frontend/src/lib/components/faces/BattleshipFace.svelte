<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { BattleshipLayout } from '#lib/types.ts';

	interface Props {
		layout: BattleshipLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const ship = (c: number) => layout?.ships.find((s) => s.cells.includes(c));
	const shot = (c: number) => !!layout?.shots.includes(c);
</script>

<p class="rules">{i18n.t('series.battleship.rules')}</p>
<div class="play square">
	<div class="sea">
		{#each { length: 25 }, c (c)}
			{@const s = ship(c)}
			<span
				class="cell"
				class:ship={!!s}
				class:sunk={revealed && s?.sunk}
				class:hit={!!s && shot(c)}
				class:miss={!s && shot(c)}
				data-cell={c}
				data-state={layout ? (s && shot(c) ? 'hit' : shot(c) ? 'miss' : '') : ''}
			>
				{#if layout && shot(c)}{s ? '💥' : '·'}{/if}
			</span>
		{/each}
	</div>
	{@render foil()}
</div>
<ul class="fleet">
	{#each layout?.ships ?? [] as s, i (i)}
		<li class:sunk={revealed && s.sunk} data-sunk={revealed && s.sunk ? '' : undefined}>
			{'■'.repeat(s.cells.length)} · {i18n.num(s.coins)}
		</li>
	{/each}
	<li>{i18n.t('ship.hit')}</li>
</ul>

<style>
	.sea {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		grid-template-rows: repeat(5, 1fr);
		gap: 3px;
		padding: 8px;
		border-radius: 14px;
		background: #1e3a8a;
	}
	.cell {
		display: grid;
		place-items: center;
		border-radius: 6px;
		background: #3b82f6;
		font-size: 20px;
		color: #dbeafe;
	}
	.cell.ship {
		background: #64748b;
	}
	.cell.sunk {
		background: #fbbf24;
	}
	.cell.miss {
		font-size: 30px;
	}
	.fleet {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 6px 14px;
		font-size: 13px;
		font-weight: 700;
	}
	.fleet li.sunk {
		color: #b45309;
	}
</style>
