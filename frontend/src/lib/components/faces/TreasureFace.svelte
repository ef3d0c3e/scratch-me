<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { api, post } from '#lib/api.ts';
	import type { DigState, Scratched, Spot, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Treasure Dig: the player picks where to dig, one spot at a time. Each
	 * spot has its own foil; scratching it asks the server to dig there, and
	 * only its answer shows what was under. The last dig pays, and shows the
	 * rest of the beach.
	 */
	interface Props {
		ticket: TicketDto;
		tool: ToolId;
		/** The last dig's answer, as a scratch answers */
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onscratch: (speed: number) => void;
		onscratchstart: () => void;
		onscratchend: () => void;
		onfind: (spot: Spot) => void;
	}
	let { ticket, tool, ondone, onerror, onscratch, onscratchstart, onscratchend, onfind }: Props =
		$props();
	const i18n = getI18n();
	const CELLS = 16;

	let spots = $state<(Spot | null)[]>(new Array(CELLS).fill(null));
	let left = $state(0);
	let ready = $state(false);
	let digging = $state<number | null>(null);
	/** After the last dig: everything, to see what was missed */
	let beach = $state<Spot[] | null>(null);

	onMount(async () => {
		try {
			const s = await api<DigState>(`/tickets/${ticket.id}/dig`);
			for (const d of s.dug) spots[d.cell] = d.spot;
			left = s.digs_left;
			ready = true;
		} catch (e) {
			onerror(e);
		}
	});

	async function dig(cell: number) {
		if (digging !== null || left === 0 || spots[cell]) return;
		digging = cell;
		try {
			const s = await post<DigState>(`/tickets/${ticket.id}/dig`, { cell });
			const found = s.dug.find((d) => d.cell === cell)?.spot ?? { kind: 'sand' };
			spots[cell] = found;
			left = s.digs_left;
			onfind(found);
			if (s.done) {
				beach = (s.done.layout as { cells: Spot[] }).cells;
				ondone(s.done);
			}
		} catch (e) {
			onerror(e);
		} finally {
			digging = null;
		}
	}
</script>

<p class="rules">{i18n.t('series.treasure_dig.rules')}</p>
<p class="left" aria-live="polite" data-testid="digs-left">
	{#if beach}
		{i18n.t('treasure.over')}
	{:else if ready}
		⛏️ {i18n.t('treasure.left', { n: left })}
	{/if}
</p>
<div class="play square">
	<div class="beach" class:over={!!beach}>
		{#each { length: CELLS }, i (i)}
			{@const s = spots[i] ?? (beach ? beach[i] : null)}
			{@const mine = spots[i] !== null}
			<div
				class="spot"
				class:dug={mine}
				class:missed={!mine && !!beach}
				data-spot={s?.kind ?? ''}
				data-cell={i}
			>
				{#if s}
					<span class="what" class:chest={s.kind === 'chest'}>
						{#if s.kind === 'chest'}
							<span aria-hidden="true">💰</span><b>{i18n.num(s.coins)}</b>
						{:else if s.kind === 'map'}
							<span aria-hidden="true">🗺️</span><small>{i18n.t('treasure.map')}</small>
						{:else}
							<span class="sand" aria-label={i18n.t('treasure.sand')}>·</span>
						{/if}
					</span>
				{/if}
				{#if ready && !s}
					<div class="cover" class:locked={left === 0 || digging !== null}>
						<ScratchFoil
							revealed={false}
							{tool}
							shiny={false}
							label="?"
							onreveal={() => dig(i)}
							{onscratch}
							{onscratchstart}
							{onscratchend}
						/>
					</div>
				{/if}
				{#if digging === i}<span class="busy" aria-hidden="true">⏳</span>{/if}
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
	.beach {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 6px;
		padding: 8px;
		border-radius: 14px;
		background: radial-gradient(circle at 30% 20%, #ffe7b8, #e9c27d);
	}
	.spot {
		position: relative;
		border-radius: 10px;
		background: #f3d8a2;
		box-shadow: inset 0 0 0 2px rgba(122, 74, 18, 0.2);
		overflow: hidden;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
		filter: grayscale(0.6);
		opacity: 0.8;
	}
	.what {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		animation: pop 0.6s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.what b {
		font-size: 13px;
		color: #7a4a12;
	}
	.what small {
		font-size: 10px;
		font-weight: 800;
		color: #7a4a12;
	}
	.dug .what.chest {
		background: radial-gradient(circle, #fff7d1, #ffd66b);
	}
	.missed .what {
		opacity: 0.45;
		animation: none;
	}
	.sand {
		color: #b88a4a;
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
