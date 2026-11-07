<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { move } from '#lib/play.ts';
	import type { FoilId, Scratched, TicketDto, ToolId } from '#lib/types.ts';
	import ScratchFoil from '../ScratchFoil.svelte';

	/**
	 * Roulette: red, black and green each hide a bet. Starting to scratch one
	 * places it (sent at once, the others lock); once uncovered, the wheel
	 * spins to the pocket the server drew.
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
	type Colour = 'red' | 'black' | 'green';
	type View = { pick?: Colour; bets?: [number, number, number]; pocket?: number; colour?: Colour };
	const COLOURS: Colour[] = ['red', 'black', 'green'];
	const RED = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
	const colourOf = (n: number): Colour => (n === 0 ? 'green' : RED.includes(n) ? 'red' : 'black');

	let pick = $state<Colour | null>(null);
	let view = $state<View>({});
	let done = $state<Scratched | null>(null);
	let uncovered = $state(false);
	let spin = $state<'idle' | 'spinning' | 'stopped'>('idle');
	/** The number on show while the wheel turns */
	let shown = $state(0);

	async function choose(colour: Colour) {
		if (pick) return;
		pick = colour;
		try {
			const s = await move<View>(p.ticket.id, { colour });
			view = s.view;
			done = s.done;
			if (uncovered) roll();
		} catch (e) {
			p.onerror(e);
		}
	}

	function reveal() {
		uncovered = true;
		if (done) roll();
	}

	/** Numbers flick by, slowing down, and stop on the pocket */
	function roll() {
		if (spin !== 'idle' || !done || view.pocket === undefined) return;
		spin = 'spinning';
		const end = view.pocket;
		let delay = 40;
		const step = () => {
			if (delay > 260) {
				shown = end;
				spin = 'stopped';
				p.onbeat(view.colour === pick);
				if (done) p.ondone(done);
				return;
			}
			shown = Math.floor(Math.random() * 37);
			p.ontick();
			delay *= 1.15;
			setTimeout(step, delay);
		};
		step();
	}
	const bet = (c: Colour) => view.bets?.[COLOURS.indexOf(c)];
</script>

<p class="rules">{i18n.t('series.roulette.rules')}</p>
<p class="say" aria-live="polite" data-testid="roulette-step">
	{#if spin === 'stopped'}
		{view.colour === pick
			? i18n.t('roulette.won', { n: view.pocket ?? 0 })
			: i18n.t('roulette.lost', { n: view.pocket ?? 0 })}
	{:else if spin === 'spinning'}
		{i18n.t('roulette.spinning')}
	{:else if pick}
		{i18n.t('roulette.placed', { colour: i18n.t(`roulette.${pick}`) })}
	{:else}
		{i18n.t('roulette.pick')}
	{/if}
</p>
<div class="wheel" aria-hidden="true">
	<span
		class="pocket {spin === 'idle' ? 'none' : colourOf(shown)}"
		class:stopped={spin === 'stopped'}
		data-pocket={spin === 'stopped' ? shown : undefined}>{spin === 'idle' ? '?' : shown}</span
	>
</div>
<div class="areas">
	{#each COLOURS as c (c)}
		{@const mine = pick === c}
		<div class="area {c}" class:mine class:other={!!pick && !mine} data-colour={c}>
			<b>{i18n.t(`roulette.${c}`)}</b>
			<small>{c === 'green' ? '1 / 37' : '18 / 37'}</small>
			<div class="cell">
				{#if bet(c) !== undefined}
					<span class="bet" class:faded={!mine}>🪙 {i18n.num(bet(c) ?? 0)}</span>
				{/if}
				{#if !(mine && uncovered) && !(pick && !mine && spin === 'stopped')}
					<div class="cover" class:locked={!!pick && !mine}>
						<ScratchFoil
							revealed={false}
							tool={p.tool}
							foil={p.foil}
							shiny={false}
							label={i18n.t(`roulette.${c}`)}
							onreveal={() => mine && reveal()}
							onscratch={p.onscratch}
							onscratchstart={() => {
								p.onscratchstart();
								choose(c);
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
	.wheel {
		display: grid;
		place-items: center;
		height: 84px;
		margin-bottom: 8px;
		border-radius: 50%;
		background: radial-gradient(circle, #6b3d00 0 40%, #3a2000 41% 100%);
		width: 84px;
		margin-inline: auto;
		box-shadow: 0 0 0 4px #c9a24a;
	}
	.pocket {
		display: grid;
		place-items: center;
		width: 48px;
		height: 48px;
		border-radius: 50%;
		font: 400 22px/1 var(--display);
		color: #fff;
		background: #555;
	}
	.pocket.red {
		background: #d4202f;
	}
	.pocket.black {
		background: #1b1530;
	}
	.pocket.green {
		background: #138a4a;
	}
	.pocket.stopped {
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.areas {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 8px;
	}
	.area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		padding: 8px 6px;
		border-radius: 14px;
		color: #fff;
		transition: opacity 0.3s;
	}
	.area.red {
		background: #d4202f;
	}
	.area.black {
		background: #1b1530;
	}
	.area.green {
		background: #138a4a;
	}
	.area.mine {
		box-shadow: 0 0 0 3px #ffc23d;
	}
	.area.other {
		opacity: 0.55;
	}
	.area small {
		font-size: 11px;
		opacity: 0.8;
	}
	.cell {
		position: relative;
		width: 100%;
		aspect-ratio: 1.2;
		margin-top: 4px;
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
	@keyframes pop {
		from {
			transform: scale(1.7);
		}
	}
</style>
