<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Keno: twenty numbers under their own foil. Each one scratched is a pick
	 * and shows at once whether it was drawn; the fifth ends it, and the whole
	 * draw shows.
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
	type View = {
		picks: { cell: number; hit: boolean }[];
		matches: number;
		left: number;
		pays: number[];
		drawn: number[] | null;
	};
	let view = $state<View | null>(null);
	let busy = $state<number | null>(null);
	const over = $derived(!!view?.drawn);

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function pick(cell: number) {
		if (busy !== null || over) return;
		busy = cell;
		try {
			const s = await move<View>(p.ticket.id, { pick: cell });
			view = s.view;
			p.onbeat(!!s.view.picks.find((x) => x.cell === cell)?.hit);
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = null;
		}
	}
	const picked = (c: number) => view?.picks.find((x) => x.cell === c);
</script>

<p class="rules">{i18n.t('series.keno.rules')}</p>
<p class="say" aria-live="polite" data-testid="keno-state">
	{#if over}
		{i18n.t('keno.matches', { n: view?.matches ?? 0 })}
	{:else if view}
		{i18n.t('keno.left', { n: view.left })} · {i18n.t('keno.matches', { n: view.matches })}
	{/if}
</p>
<div class="play wide">
	<div class="board">
		{#each { length: 20 }, c (c)}
			{@const x = picked(c)}
			{@const drawn = !!view?.drawn?.includes(c)}
			<div
				class="cell"
				class:hit={x?.hit}
				class:miss={x && !x.hit}
				class:drawn={!x && drawn}
				data-cell={c}
				data-state={x ? (x.hit ? 'hit' : 'miss') : drawn ? 'drawn' : ''}
			>
				<span class="n">{c + 1}</span>
				{#if x}<span class="mark">{x.hit ? '✔' : '✖'}</span>{/if}
				{#if !x && !over && view}
					<div class="cover" class:locked={busy !== null}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label={String(c + 1)}
							onreveal={() => pick(c)}
							onscratch={p.onscratch}
							onscratchstart={p.onscratchstart}
							onscratchend={p.onscratchend}
						/>
					</div>
				{/if}
			</div>
		{/each}
	</div>
</div>
{#if view}
	<ul class="paytable" aria-label={i18n.t('paytable.title')}>
		{#each view.pays as coins, n (n)}
			{#if coins > 0}
				<li class:on={over && view.matches === n}>
					{i18n.t('keno.matches', { n })} = {i18n.num(coins)}
				</li>
			{/if}
		{/each}
	</ul>
{/if}

<style>
	.say {
		text-align: center;
		font-weight: 800;
		min-height: 1.4em;
		margin: -4px 0 8px;
	}
	.board {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		grid-template-rows: repeat(4, minmax(0, 1fr));
		gap: 4px;
		padding: 8px;
		border-radius: 14px;
		background: #7c2d12;
	}
	.cell {
		position: relative;
		min-width: 0;
		min-height: 0;
		border-radius: 8px;
		background: #fff7ed;
		overflow: hidden;
		display: grid;
		place-items: center;
	}
	.n {
		font: 400 20px/1 var(--display);
		color: #7c2d12;
	}
	.mark {
		position: absolute;
		top: 2px;
		right: 5px;
		font-size: 14px;
		font-weight: 800;
	}
	.cell.hit {
		background: #bbf7d0;
	}
	.cell.hit .mark {
		color: #15803d;
	}
	.cell.miss {
		background: #fecaca;
	}
	.cell.miss .mark {
		color: #b91c1c;
	}
	.cell.drawn {
		box-shadow: inset 0 0 0 3px #f59e0b;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
	}
	.paytable {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 4px 14px;
		font-size: 12.5px;
		font-weight: 700;
	}
	.paytable li.on {
		color: var(--edge-ink);
	}
</style>
