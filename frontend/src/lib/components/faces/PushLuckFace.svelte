<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/** Twelve panels with their own foil: each one scratched is flipped; cash out any time */
	interface Props {
		ticket: TicketDto;
		tool: ToolId;
		foil: FoilId;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onscratchstart: () => void;
		onscratch: (speed: number) => void;
		onscratchend: () => void;
		onbeat: (good: boolean) => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type Panel = { kind: 'coins'; coins: number } | { kind: 'bomb' };
	type View = {
		flipped: { panel: number; is: Panel }[];
		pot: number;
		boom: boolean;
		panels: Panel[] | null;
	};

	let view = $state<View | null>(null);
	let busy = $state(false);
	const shown = (i: number) =>
		view?.flipped.find((f) => f.panel === i)?.is ?? view?.panels?.[i] ?? null;
	const mine = (i: number) => !!view?.flipped.some((f) => f.panel === i);
	const over = $derived(!!view?.panels);

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function act(mv: unknown) {
		if (busy) return;
		busy = true;
		try {
			const s = await move<View>(p.ticket.id, mv);
			view = s.view;
			const last = s.view.flipped.at(-1)?.is;
			if (last) p.onbeat(last.kind === 'coins');
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
</script>

<p class="rules">{i18n.t('series.push_luck.rules')}</p>
<div class="pot" data-testid="pot">
	{#if view?.boom}
		💥 {i18n.t('push.boom')}
	{:else}
		🪙 {i18n.t('push.pot', { n: view?.pot ?? 0 })}
	{/if}
</div>
<div class="play wide">
	<div class="panels">
		{#each { length: 12 }, i (i)}
			{@const s = shown(i)}
			<div
				class="panel"
				class:mine={mine(i)}
				class:missed={!mine(i) && !!s}
				data-panel={i}
				data-is={s?.kind ?? ''}
			>
				{#if s}
					<span class="what" class:bomb={s.kind === 'bomb'}
						>{s.kind === 'bomb' ? '💣' : `+${i18n.num(s.coins)}`}</span
					>
				{:else if view && !over}
					<div class="cover" class:locked={busy}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label="?"
							onreveal={() => act({ kind: 'flip', panel: i })}
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
{#if view && !over}
	<button
		type="button"
		class="btn primary cash"
		disabled={busy || view.flipped.length === 0}
		onclick={() => act({ kind: 'cash' })}>{i18n.t('push.cash', { n: view.pot })}</button
	>
{/if}

<style>
	.pot {
		text-align: center;
		font: 400 22px/1 var(--display);
		color: #c2410c;
		margin: -4px 0 10px;
	}
	.panels {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		grid-template-rows: repeat(3, 1fr);
		gap: 6px;
		padding: 8px;
	}
	.panel {
		position: relative;
		border-radius: 10px;
		background: var(--paper);
		overflow: hidden;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
	}
	.what {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font: 400 18px/1 var(--display);
		color: #c2410c;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.what.bomb {
		font-size: 28px;
		background: #fecaca;
	}
	.missed .what {
		opacity: 0.4;
		animation: none;
	}
	.cash {
		width: 100%;
		margin-top: 12px;
	}
	@keyframes pop {
		from {
			transform: scale(0.4);
			opacity: 0;
		}
	}
</style>
