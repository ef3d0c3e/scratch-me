<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { move } from '#lib/play.ts';
	import type { FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Coin Flip: Heads and Tails each hide a bet. Starting to scratch one is
	 * the call (sent to the server at once, the other cell locks); once it is
	 * uncovered, the coin flips. The server drew the bets and the flip.
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
	type Side = 'heads' | 'tails';
	type View = { call?: Side; bets?: [number, number]; flip?: Side };
	const SIDES: Side[] = ['heads', 'tails'];

	let call = $state<Side | null>(null);
	let view = $state<View>({});
	let done = $state<Scratched | null>(null);
	let uncovered = $state(false);
	/** The coin in the air, then landed */
	let flip = $state<'idle' | 'spinning' | 'landed'>('idle');

	async function choose(side: Side) {
		if (call) return;
		call = side;
		try {
			const s = await move<View>(p.ticket.id, { side });
			view = s.view;
			done = s.done;
			if (uncovered) toss();
		} catch (e) {
			p.onerror(e);
		}
	}

	function reveal() {
		uncovered = true;
		if (done) toss();
	}

	function toss() {
		if (flip !== 'idle' || !done) return;
		flip = 'spinning';
		setTimeout(() => {
			flip = 'landed';
			p.onbeat(view.flip === call);
			if (done) p.ondone(done);
		}, 1400);
	}
	const bet = (side: Side) => view.bets?.[SIDES.indexOf(side)];
</script>

<p class="rules">{i18n.t('series.coin_flip.rules')}</p>
<p class="say" aria-live="polite" data-testid="coin-step">
	{#if flip === 'landed'}
		{view.flip === call
			? i18n.t('coin.won', { side: i18n.t(`coin.${view.flip ?? 'heads'}`) })
			: i18n.t('coin.lost', { side: i18n.t(`coin.${view.flip ?? 'heads'}`) })}
	{:else if flip === 'spinning'}
		{i18n.t('coin.flipping')}
	{:else if call}
		{i18n.t('coin.called', { side: i18n.t(`coin.${call}`) })}
	{:else}
		{i18n.t('coin.pick')}
	{/if}
</p>
<div class="table">
	{#each SIDES as side (side)}
		{@const mine = call === side}
		<div class="side" class:mine class:other={!!call && !mine} data-side={side}>
			<span class="face" aria-hidden="true">{side === 'heads' ? '👑' : '🦅'}</span>
			<b>{i18n.t(`coin.${side}`)}</b>
			<div class="cell">
				{#if bet(side) !== undefined}
					<span class="bet" class:faded={!mine}>🪙 {i18n.num(bet(side) ?? 0)}</span>
				{/if}
				{#if !(mine && uncovered) && !(call && !mine && flip === 'landed')}
					<div class="cover" class:locked={!!call && !mine}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label={i18n.t(`coin.${side}`)}
							onreveal={() => mine && reveal()}
							onscratch={p.onscratch}
							onscratchstart={() => {
								p.onscratchstart();
								choose(side);
							}}
							onscratchend={p.onscratchend}
						/>
					</div>
				{/if}
			</div>
		</div>
	{/each}
</div>
<div class="coin-wrap" aria-hidden="true">
	<span
		class="coin"
		class:spinning={flip === 'spinning'}
		class:landed={flip === 'landed'}
		data-flip={flip === 'landed' ? view.flip : undefined}
		>{flip === 'landed' ? (view.flip === 'heads' ? '👑' : '🦅') : '🪙'}</span
	>
</div>

<style>
	.say {
		text-align: center;
		font-weight: 800;
		min-height: 1.4em;
		margin: -4px 0 8px;
	}
	.table {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.side {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 10px;
		border-radius: 16px;
		background: #f6deae;
		transition: opacity 0.3s;
	}
	.side.mine {
		box-shadow: 0 0 0 3px var(--edge);
	}
	.side.other {
		opacity: 0.6;
	}
	.face {
		font-size: 28px;
	}
	.cell {
		position: relative;
		width: 100%;
		aspect-ratio: 1.4;
		border-radius: 12px;
		background: #fffaf0;
		display: grid;
		place-items: center;
		overflow: hidden;
	}
	.bet {
		font: 400 26px/1 var(--display);
		color: #7a4a12;
	}
	.bet.faded {
		opacity: 0.5;
		font-size: 20px;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
		filter: grayscale(0.7);
	}
	.coin-wrap {
		display: grid;
		place-items: center;
		height: 72px;
		perspective: 400px;
	}
	.coin {
		font-size: 44px;
	}
	.coin.spinning {
		animation: spin 0.35s linear infinite;
	}
	.coin.landed {
		animation: land 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes spin {
		from {
			transform: translateY(-14px) rotateX(0);
		}
		to {
			transform: translateY(-14px) rotateX(360deg);
		}
	}
	@keyframes land {
		from {
			transform: scale(1.6);
		}
	}
</style>
