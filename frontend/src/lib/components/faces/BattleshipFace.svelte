<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { BattleshipView, FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Battleship: every cell of the sea has its own little foil. Scratching
	 * one is a shot; the server answers hit or miss. Sunk ships show whole,
	 * the rest of the fleet once the shots are spent.
	 */
	interface Props {
		ticket: TicketDto;
		tool: ToolId;
		foil: FoilId;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onscratch: (speed: number) => void;
		onscratchstart: () => void;
		onscratchend: () => void;
		onbeat: (good: boolean) => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	const CELLS = 25;

	let view = $state<BattleshipView | null>(null);
	let over = $state(false);
	let firing = $state<number | null>(null);

	onMount(() => {
		look<BattleshipView>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function fire(cell: number) {
		if (firing !== null || over || !view || shot(cell)) return;
		firing = cell;
		try {
			const s = await move<BattleshipView>(p.ticket.id, { cell });
			const before = view.sunk;
			view = s.view;
			const hit = !!s.view.shots.find((x) => x.cell === cell)?.hit;
			if (hit || s.view.sunk > before) p.onbeat(true);
			if (s.done) {
				over = true;
				p.ondone(s.done);
			}
		} catch (e) {
			p.onerror(e);
		} finally {
			firing = null;
		}
	}
	const shot = (c: number) => view?.shots.find((s) => s.cell === c);
	const ship = (c: number) => view?.ships.find((s) => s.cells.includes(c));
</script>

<p class="rules">{i18n.t('series.battleship.rules')}</p>
<p class="left" aria-live="polite" data-testid="shots-left">
	{#if over}
		{i18n.t('battleship.over', { n: view?.sunk ?? 0 })}
	{:else if view}
		🎯 {i18n.t('battleship.left', { n: view.left })}
		{#if view.coins > 0}· 🪙 {i18n.num(view.coins)}{/if}
	{/if}
</p>
<div class="play square">
	<div class="sea" class:over>
		{#each { length: CELLS }, c (c)}
			{@const s = shot(c)}
			{@const boat = ship(c)}
			<div
				class="cell"
				class:hit={s?.hit}
				class:miss={s && !s.hit}
				class:ship={!!boat}
				class:missed={!s && !!boat}
				data-cell={c}
				data-state={s ? (s.hit ? 'hit' : 'miss') : ''}
			>
				{#if s}
					<span class="mark">{s.hit ? '💥' : '·'}</span>
				{:else if view && !over}
					<div class="cover" class:locked={firing !== null}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label="?"
							onreveal={() => fire(c)}
							onscratch={p.onscratch}
							onscratchstart={p.onscratchstart}
							onscratchend={p.onscratchend}
						/>
					</div>
				{/if}
				{#if firing === c}<span class="busy" aria-hidden="true">⏳</span>{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.left {
		text-align: center;
		font-weight: 800;
		margin: -4px 0 8px;
		min-height: 1.4em;
	}
	/* Every cell the same size, whatever it shows: a foil, a hit, a miss */
	.sea {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		grid-template-rows: repeat(5, minmax(0, 1fr));
		gap: 4px;
		padding: 8px;
		border-radius: 14px;
		background: linear-gradient(#1d4ed8, #0b1640);
	}
	.cell {
		position: relative;
		min-width: 0;
		min-height: 0;
		border-radius: 6px;
		background: #3b82f6;
		overflow: hidden;
		display: grid;
		place-items: center;
	}
	.cell.ship {
		background: #64748b;
		box-shadow: inset 0 0 0 2px #334155;
	}
	.cell.hit {
		background: #fca5a5;
	}
	.cell.ship.hit {
		background: #ef4444;
	}
	.cell.miss {
		background: #93c5fd;
	}
	.cell.missed {
		opacity: 0.6;
	}
	.mark {
		font-size: 20px;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.miss .mark {
		color: #1e3a8a;
		font-size: 28px;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
	}
	.busy {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
	}
	@keyframes pop {
		from {
			transform: scale(0.4);
			opacity: 0;
		}
	}
</style>
