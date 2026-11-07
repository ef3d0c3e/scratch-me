<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { move } from '#lib/play.ts';
	import type { FoilId, PlayingCard as Card, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';
	import PlayingCard from '../PlayingCard.svelte';

	/**
	 * Baccarat: Player, Banker and Tie each hide a bet. Starting to scratch one
	 * is the call (sent at once, the others lock); once uncovered, the hands are
	 * dealt card by card, as the server dealt them.
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
		ontick: () => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type Side = 'player' | 'banker' | 'tie';
	type View = {
		call?: Side;
		bets?: [number, number, number];
		player?: Card[];
		banker?: Card[];
		winner?: Side;
		totals?: [number, number];
	};
	const SIDES: Side[] = ['player', 'banker', 'tie'];

	let call = $state<Side | null>(null);
	let view = $state<View>({});
	let done = $state<Scratched | null>(null);
	let uncovered = $state(false);
	/** Cards dealt so far, in the order of the coup: P, B, P, B, then the thirds */
	let dealt = $state(0);
	let deal = $state<'idle' | 'dealing' | 'over'>('idle');

	async function choose(side: Side) {
		if (call) return;
		call = side;
		try {
			const s = await move<View>(p.ticket.id, { side });
			view = s.view;
			done = s.done;
			if (uncovered) start();
		} catch (e) {
			p.onerror(e);
		}
	}

	function reveal() {
		uncovered = true;
		if (done) start();
	}

	const all = $derived((view.player?.length ?? 0) + (view.banker?.length ?? 0));
	function start() {
		if (deal !== 'idle' || !done) return;
		deal = 'dealing';
		const step = () => {
			dealt++;
			p.ontick();
			if (dealt < all) {
				setTimeout(step, 450);
				return;
			}
			setTimeout(() => {
				deal = 'over';
				p.onbeat(view.winner === call);
				if (done) p.ondone(done);
			}, 500);
		};
		setTimeout(step, 200);
	}
	/** The coup's order: player 1, banker 1, player 2, banker 2, player 3, banker 3 */
	const shown = (hand: 'player' | 'banker', i: number) => {
		const order = i < 2 ? i * 2 + (hand === 'banker' ? 1 : 0) : hand === 'player' ? 4 : all - 1;
		return order < dealt;
	};
	const bet = (s: Side) => view.bets?.[SIDES.indexOf(s)];
</script>

<p class="rules">{i18n.t('series.baccarat.rules')}</p>
<p class="say" aria-live="polite" data-testid="baccarat-step">
	{#if deal === 'over'}
		{view.winner === call
			? i18n.t('baccarat.won', { side: i18n.t(`baccarat.${view.winner ?? 'tie'}`) })
			: i18n.t('baccarat.lost', { side: i18n.t(`baccarat.${view.winner ?? 'tie'}`) })}
	{:else if deal === 'dealing'}
		{i18n.t('baccarat.dealing')}
	{:else if call}
		{i18n.t('baccarat.called', { side: i18n.t(`baccarat.${call}`) })}
	{:else}
		{i18n.t('baccarat.pick')}
	{/if}
</p>
{#if deal !== 'idle'}
	<div class="felt">
		{#each ['player', 'banker'] as const as hand (hand)}
			{@const cards = view[hand] ?? []}
			<div class="hand" class:won={deal === 'over' && view.winner === hand} data-hand={hand}>
				<b>{i18n.t(`baccarat.${hand}`)}</b>
				<span class="cards">
					{#each cards as c, i (i)}
						{#if shown(hand, i)}<span class="deal"><PlayingCard card={c} /></span>{/if}
					{/each}
				</span>
				{#if deal === 'over'}<span class="total">{view.totals?.[hand === 'player' ? 0 : 1]}</span
					>{/if}
			</div>
		{/each}
	</div>
{/if}
<div class="areas">
	{#each SIDES as s (s)}
		{@const mine = call === s}
		<div class="area {s}" class:mine class:other={!!call && !mine} data-side={s}>
			<b>{i18n.t(`baccarat.${s}`)}</b>
			<div class="cell">
				{#if bet(s) !== undefined}
					<span class="bet" class:faded={!mine}>🪙 {i18n.num(bet(s) ?? 0)}</span>
				{/if}
				{#if !(mine && uncovered) && !(call && !mine && deal === 'over')}
					<div class="cover" class:locked={!!call && !mine}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label={i18n.t(`baccarat.${s}`)}
							onreveal={() => mine && reveal()}
							onscratch={p.onscratch}
							onscratchstart={() => {
								p.onscratchstart();
								choose(s);
							}}
							onscratchend={p.onscratchend}
						/>
					</div>
				{/if}
			</div>
		</div>
	{/each}
</div>

<style>
	.say {
		text-align: center;
		font-weight: 800;
		min-height: 1.4em;
		margin: -4px 0 8px;
	}
	.felt {
		display: grid;
		gap: 6px;
		padding: 10px;
		margin-bottom: 10px;
		border-radius: 14px;
		background: radial-gradient(ellipse at 50% 30%, #2e8a5a, #17573a);
		color: #fff;
	}
	.hand {
		display: grid;
		grid-template-columns: 70px 1fr 32px;
		align-items: center;
		gap: 8px;
		min-height: 84px;
		padding: 2px 6px;
		border-radius: 10px;
	}
	.hand.won {
		background: rgba(255, 214, 107, 0.25);
		box-shadow: inset 0 0 0 2px #ffd66b;
	}
	.cards {
		display: flex;
		gap: 4px;
	}
	.deal {
		animation: deal 0.35s ease-out both;
	}
	.total {
		font: 400 26px/1 var(--display);
		color: #ffd66b;
		text-align: center;
	}
	.areas {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 8px;
	}
	.area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 8px 6px;
		border-radius: 14px;
		color: #fff;
		transition: opacity 0.3s;
	}
	.area.player {
		background: #1d4ed8;
	}
	.area.banker {
		background: #b91c1c;
	}
	.area.tie {
		background: #15803d;
	}
	.area.mine {
		box-shadow: 0 0 0 3px #ffc23d;
	}
	.area.other {
		opacity: 0.55;
	}
	.cell {
		position: relative;
		width: 100%;
		aspect-ratio: 1.2;
		border-radius: 10px;
		background: #fffaf0;
		display: grid;
		place-items: center;
		overflow: hidden;
	}
	.bet {
		font: 400 18px/1 var(--display);
		color: #7a4a12;
	}
	.bet.faded {
		opacity: 0.5;
		font-size: 15px;
	}
	.cover {
		position: absolute;
		inset: 0;
	}
	.cover.locked {
		pointer-events: none;
		filter: grayscale(0.7);
	}
	@keyframes deal {
		from {
			transform: translateY(-20px) rotate(-8deg);
			opacity: 0;
		}
	}
</style>
