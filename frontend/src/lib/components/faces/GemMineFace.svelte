<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Gem Mine: each cell has its own foil. Scratching one digs it: a gem
	 * raises what cashing out pays, a mine loses it all. Cash out any time.
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
	type View = {
		dug: { cell: number; mine: boolean }[];
		gems: number;
		now: number;
		next: number;
		boom: boolean;
		cashed: boolean;
		mines: number[] | null;
	};
	let view = $state<View | null>(null);
	let busy = $state<number | 'cash' | null>(null);
	const over = $derived(!!view?.mines);

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function act(mv: unknown, what: number | 'cash') {
		if (busy !== null || over) return;
		busy = what;
		try {
			const s = await move<View>(p.ticket.id, mv);
			view = s.view;
			if (typeof what === 'number') p.onbeat(!s.view.boom);
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = null;
		}
	}
	const dug = (c: number) => view?.dug.find((d) => d.cell === c);
</script>

<p class="rules">{i18n.t('series.gem_mine.rules')}</p>
<div class="bar" aria-live="polite">
	{#if view?.boom}
		<b class="lost" data-testid="mine-state">{i18n.t('mine.boom')}</b>
	{:else if view?.cashed}
		<b data-testid="mine-state">{i18n.t('mine.cashed', { n: view.now })}</b>
	{:else if view}
		<span data-testid="mine-state">💎 {view.gems} · {i18n.t('mine.next', { n: view.next })}</span>
		<button
			type="button"
			class="btn small primary"
			disabled={view.gems === 0 || busy !== null}
			onclick={() => act({ kind: 'cash' }, 'cash')}>{i18n.t('mine.cash', { n: view.now })}</button
		>
	{/if}
</div>
<div class="play square">
	<div class="mine" class:over>
		{#each { length: CELLS }, c (c)}
			{@const d = dug(c)}
			{@const hidden = !!view?.mines?.includes(c)}
			<div
				class="cell"
				class:gem={d && !d.mine}
				class:boom={d?.mine}
				class:missed={!d && over}
				data-cell={c}
				data-state={d ? (d.mine ? 'mine' : 'gem') : ''}
			>
				{#if d}
					<span class="what">{d.mine ? '💥' : '💎'}</span>
				{:else if over}
					<span class="what faint">{hidden ? '💣' : '💎'}</span>
				{:else if view}
					<div class="cover" class:locked={busy !== null}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label="?"
							onreveal={() => act({ kind: 'dig', cell: c }, c)}
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

<style>
	.bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		min-height: 44px;
		margin: -4px 0 8px;
		font-weight: 800;
	}
	.lost {
		color: #b91c1c;
	}
	.mine {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		grid-template-rows: repeat(5, minmax(0, 1fr));
		gap: 4px;
		padding: 8px;
		border-radius: 14px;
		background: linear-gradient(#57534e, #292524);
	}
	.cell {
		position: relative;
		min-width: 0;
		min-height: 0;
		border-radius: 8px;
		background: #78716c;
		overflow: hidden;
		display: grid;
		place-items: center;
	}
	.cell.gem {
		background: #a5f3fc;
	}
	.cell.boom {
		background: #f87171;
	}
	.what {
		font-size: 22px;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.what.faint {
		opacity: 0.45;
		animation: none;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
	}
	@keyframes pop {
		from {
			transform: scale(0.4);
			opacity: 0;
		}
	}
</style>
