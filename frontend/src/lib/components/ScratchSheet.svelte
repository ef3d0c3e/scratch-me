<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import type { Bonus, FoilId, Scratched, ThemeId, TicketDto, ToolId } from '#lib/types.ts';
	import { show, tier as tierOf } from '#lib/fx/celebrate.ts';
	import { prizeCoins, prizeText } from '#lib/prize.ts';
	import { getFx } from '#lib/fx/fx.svelte.ts';
	import { THEME_PITCH, TOOLS } from '#lib/fx/tools.ts';
	import { FOIL_GRID, SERIES_LOOK, cosmeticKey, isPlayed } from '#lib/series.ts';
	import { PLAYED_FACES, SCRATCH_FACES } from './faces/registry.ts';
	import BonusBox from './BonusBox.svelte';
	import Celebration from './Celebration.svelte';
	import Counter from './Counter.svelte';
	import ScratchFoil from './ScratchFoil.svelte';

	/**
	 * One ticket, full screen. The server uncovers it as the sheet opens (the
	 * prize is credited then); the player sees it as they scratch the foil
	 * off. Each series draws its own face; the prize shows once the face has
	 * settled (at once, or after the Cash Wheel stops).
	 */
	interface Props {
		ticket: TicketDto;
		/** Tickets left after this one */
		left: number;
		tool?: ToolId;
		theme?: ThemeId;
		foil?: FoilId;
		/** The ticket is spent: called once, when the prize shows or the sheet closes */
		onresult: (r: Scratched) => void;
		/** The ticket was already scratched or is gone (another tab, another device) */
		ongone?: (id: number) => void;
		onnext: () => void;
		onclose: () => void;
	}
	let {
		ticket,
		left,
		tool = 'coin',
		foil = 'silver',
		theme = 'midnight',
		onresult,
		ongone,
		onnext,
		onclose
	}: Props = $props();
	const i18n = getI18n();
	const fx = getFx();

	let result = $state<Scratched | null>(null);
	let error = $state('');
	let revealed = $state(false);
	let settled = $state(false);
	let reported = false;
	let play = $state<HTMLElement>();
	/** Where the confetti bursts from, set as the prize shows */
	let origin = $state<{ x: number; y: number } | null>(null);
	let takeover = $state(false);

	onMount(() => {
		// A dig ticket is played spot by spot: its face asks the server itself
		if (!isPlayed(ticket.series)) scratch();
		// The page behind must not scroll while scratching
		const overflow = document.body.style.overflow;
		document.body.style.overflow = 'hidden';
		return () => {
			fx.scratchStop();
			document.body.style.overflow = overflow;
		};
	});

	function scratch() {
		post<Scratched>(`/tickets/${ticket.id}/scratch`)
			.then((r) => (result = r))
			.catch((e) => {
				const code = (e as { code?: string })?.code;
				// Out of date: the stack held a ticket that is no longer there to scratch
				if (code === 'already_scratched' || code === 'ticket') {
					ongone?.(ticket.id);
					error = i18n.t('scratch.gone');
				} else error = i18n.error(e);
			});
	}

	/** The last dig: the beach is out, the prize shows */
	function dug(r: Scratched) {
		result = r;
		revealed = true;
		// A beat to see the last spot before the prize lands
		setTimeout(settle, 500);
	}

	function report() {
		if (result && !reported) {
			reported = true;
			onresult(result);
		}
	}

	function reveal() {
		if (revealed) return;
		// Reveal all is a tap: audio may start if no scratch did
		fx.unlock();
		fx.scratchStop();
		revealed = true;
	}

	/** The face is done: the prize shows */
	function settle() {
		if (settled) return;
		settled = true;
		report();
		const box = play?.getBoundingClientRect();
		if (box) origin = { x: box.left + box.width / 2, y: box.top + box.height / 3 };
		fx.win(tier);
		if (s.takeover) {
			takeover = true;
			setTimeout(() => (takeover = false), 3400);
		}
	}

	function close() {
		// Closing early still spends the ticket: the lobby must count its prize
		report();
		onclose();
	}

	const look = $derived(SERIES_LOOK[ticket.series]);
	const ScratchFace = $derived(SCRATCH_FACES[ticket.series]);
	const PlayedFace = $derived(PLAYED_FACES[ticket.series]);
	/** The bonus box, once the ticket is uncovered (the server sends it with the layout) */
	const bonus = $derived((result?.layout as { bonus?: Bonus } | undefined)?.bonus ?? null);
	const grid = $derived(FOIL_GRID[ticket.series]);
	const win = $derived(result && result.prize.kind !== 'nothing');
	const tier = $derived(result ? tierOf(result.prize) : 'none');
	const s = $derived(show(tier));
	/** The words around the counting number: "+", " coins!" */
	/** Coins the prize pays, bundled ones included */
	const coinsWon = $derived(result ? prizeCoins(result.prize) : 0);
	const coinWords = $derived(i18n.t('scratch.prize.coins', { n: '\u0000' }).split('\u0000'));
	const voice = $derived({ pitch: TOOLS[tool].pitch * THEME_PITCH[theme], q: TOOLS[tool].q });

	const sub = $derived.by(() => {
		if (!result) return '';
		const p = result.prize;
		if (p.kind === 'bundle')
			return p.items
				.filter((x) => x.kind !== 'coins')
				.map((x) => '+ ' + prizeText(i18n, x))
				.join(' ');
		if (p.kind === 'tickets') return p.series.map((x) => i18n.t(`series.${x}`)).join(' · ');
		if (p.kind === 'cosmetic')
			return result.duplicate
				? i18n.t('scratch.prize.duplicate', { n: 800 })
				: i18n.t(cosmeticKey(p.item));
		return i18n.t(`scratch.prize.${p.kind}_sub`);
	});
</script>

{#snippet cover()}
	<ScratchFoil
		{revealed}
		{tool}
		{foil}
		onreveal={reveal}
		onscratchstart={() => fx.unlock()}
		onscratch={(speed) => fx.scratch(speed, voice.pitch, voice.q)}
		onscratchend={() => fx.scratchStop()}
		label={i18n.t('scratch.foil')}
		cols={grid.cols}
		rows={grid.rows}
	/>
{/snippet}

<div
	class="sheet"
	class:shake={settled && s.shake}
	role="dialog"
	aria-modal="true"
	aria-label={i18n.t(`series.${ticket.series}`)}
	data-series={ticket.series}
>
	<header>
		<span class="badge">{i18n.t(`source.${ticket.source}`)}</span>
		<button type="button" class="close" onclick={close} aria-label={i18n.t('scratch.close')}
			>✕</button
		>
	</header>

	<article
		class="ticket"
		class:win={settled && win}
		class:golden={ticket.series === 'golden'}
		style:--edge={look.edge}
		style:--edge-ink={look.ink}
		style:--edge-shadow={look.shadow}
		bind:this={play}
	>
		<div class="head">
			<h2>{i18n.t(`series.${ticket.series}`)}</h2>
		</div>
		{#if PlayedFace}
			<PlayedFace
				{ticket}
				{tool}
				{foil}
				ondone={dug}
				onerror={(e: unknown) => (error = i18n.error(e))}
				onscratchstart={() => fx.unlock()}
				onscratch={(speed: number) => fx.scratch(speed, voice.pitch, voice.q)}
				onscratchend={() => fx.scratchStop()}
				onfind={(spot: { kind: string }) => {
					fx.scratchStop();
					if (spot.kind !== 'sand') fx.win('small');
				}}
				onbeat={(good: boolean) => (good ? fx.win('small') : fx.win('none'))}
				ontick={() => fx.tick()}
			/>
		{:else if ScratchFace}
			<ScratchFace
				layout={result?.layout ?? null}
				{revealed}
				foil={cover}
				onsettled={settle}
				ontick={() => fx.tick()}
			/>
		{/if}
		{#if bonus}
			<BonusBox {bonus} {revealed} {tool} {foil} />
		{/if}
	</article>

	<div class="outcome" aria-live="polite">
		{#if error}
			<p class="err" role="alert">{error}</p>
		{:else if settled && result}
			<p class="title" class:win data-tier={tier}>
				{#if result.prize.kind === 'coins' || (result.prize.kind === 'bundle' && coinsWon > 0)}
					{coinWords[0]}<Counter value={coinsWon} from={0} duration={s.count} />{coinWords[1]}
				{:else if result.prize.kind === 'tickets'}
					{i18n.t('scratch.prize.tickets', { n: result.prize.series.length })}
				{:else if result.prize.kind === 'cosmetic'}
					{i18n.t(`scratch.prize.cosmetic_${result.prize.item.kind}`)}
				{:else}
					{i18n.t(`scratch.prize.${result.prize.kind}`)}
				{/if}
			</p>
			<p class="sub">{sub}</p>
		{:else if !revealed}
			<p class="hint">
				{ticket.series === 'treasure_dig'
					? i18n.t('treasure.hint')
					: isPlayed(ticket.series)
						? ''
						: i18n.t('scratch.hint')}
			</p>
		{/if}
	</div>

	<footer>
		{#if error}
			<button type="button" class="btn primary" onclick={onclose}>{i18n.t('scratch.done')}</button>
		{:else if !revealed && isPlayed(ticket.series)}
			<!-- No reveal all: the player chooses where to dig -->
		{:else if !revealed}
			<button type="button" class="btn ghost" disabled={!result} onclick={reveal}
				>{i18n.t('scratch.reveal_all')}</button
			>
		{:else if !settled}
			<!-- The wheel is still turning -->
		{:else if left > 0}
			<button type="button" class="btn primary" onclick={onnext}
				>{i18n.t('scratch.next', { n: left })}</button
			>
			<button type="button" class="btn ghost" onclick={onclose}>{i18n.t('scratch.done')}</button>
		{:else}
			<button type="button" class="btn primary" onclick={onclose}>{i18n.t('scratch.done')}</button>
		{/if}
	</footer>
</div>

{#if origin && tier !== 'none'}
	<Celebration {tier} {origin} />
{/if}

{#if takeover && result?.prize.kind === 'coins'}
	<button type="button" class="takeover" onclick={() => (takeover = false)}>
		<span class="rays" aria-hidden="true"></span>
		<span class="jackpot">{i18n.t('scratch.jackpot')}</span>
		<span class="amount">+<Counter value={result.prize.amount} from={0} duration={s.count} /></span>
		<span class="tap">{i18n.t('scratch.tap_to_continue')}</span>
	</button>
{/if}

<style>
	.sheet {
		/* Dragging a finger scratches; it must not select the text under it */
		user-select: none;
		-webkit-user-select: none;
		-webkit-touch-callout: none;
		position: fixed;
		inset: 0;
		z-index: 50;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 14px;
		padding: calc(12px + var(--safe-top)) 16px calc(16px + var(--safe-bottom));
		background:
			radial-gradient(90% 50% at 50% 30%, rgba(255, 194, 61, 0.18), transparent 70%),
			rgba(14, 7, 34, 0.96);
		overflow-y: auto;
		animation: rise 0.35s cubic-bezier(0.2, 0.9, 0.3, 1.1) both;
	}
	.sheet.shake {
		animation: shake 0.5s cubic-bezier(0.36, 0.07, 0.19, 0.97) 0.1s both;
	}
	@keyframes shake {
		10%,
		90% {
			transform: translate(-2px, 1px);
		}
		20%,
		80% {
			transform: translate(4px, -2px);
		}
		30%,
		50%,
		70% {
			transform: translate(-7px, 2px) rotate(-0.6deg);
		}
		40%,
		60% {
			transform: translate(7px, -1px) rotate(0.6deg);
		}
	}
	.takeover {
		position: fixed;
		inset: 0;
		z-index: 60;
		border: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		overflow: hidden;
		color: var(--ink);
		background: radial-gradient(circle, rgba(61, 31, 134, 0.92), rgba(10, 5, 26, 0.97) 70%);
		animation: flash 0.5s ease-out both;
	}
	@keyframes flash {
		from {
			background-color: #fff6e5;
			opacity: 0;
		}
	}
	.rays {
		position: absolute;
		inset: -50%;
		background: repeating-conic-gradient(
			from 0deg,
			rgba(255, 194, 61, 0.22) 0deg 10deg,
			transparent 10deg 20deg
		);
		animation: spin 12s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.jackpot {
		position: relative;
		font: 400 clamp(44px, 14vw, 84px) / 1 var(--display);
		color: var(--gold);
		text-shadow:
			0 5px 0 #9a5a00,
			0 0 40px rgba(255, 194, 61, 0.8);
		animation: slam 0.7s cubic-bezier(0.3, 1.8, 0.5, 1) 0.15s both;
	}
	@keyframes slam {
		from {
			transform: scale(3) rotate(-8deg);
			opacity: 0;
		}
	}
	.amount {
		position: relative;
		font: 800 40px/1 var(--body);
		animation: bounce 0.6s cubic-bezier(0.3, 1.8, 0.5, 1) 0.5s both;
	}
	.tap {
		position: relative;
		color: var(--muted);
		font-size: 14px;
		margin-top: 24px;
		animation: fade-in 0.4s ease 2s both;
	}
	@keyframes fade-in {
		from {
			opacity: 0;
		}
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(40px) scale(0.96);
		}
	}
	header {
		width: min(420px, 100%);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.badge {
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--muted);
		border: 1px solid var(--line);
		border-radius: 999px;
		padding: 4px 10px;
	}
	.close {
		width: 44px;
		height: 44px;
		border-radius: 50%;
		border: 0;
		background: var(--night-3);
		font-size: 18px;
	}
	.ticket {
		width: min(420px, 100%);
		background:
			radial-gradient(circle at 0 50%, transparent 10px, var(--paper-2) 10.5px) left / 51% 100%
				no-repeat,
			radial-gradient(circle at 100% 50%, transparent 10px, var(--paper-2) 10.5px) right / 51% 100%
				no-repeat;
		color: var(--paper-ink);
		border-radius: 18px;
		padding: 14px 16px 16px;
		box-shadow:
			0 0 0 4px var(--edge),
			0 24px 60px -20px rgba(0, 0, 0, 0.8);
		transition: box-shadow 0.4s ease;
	}
	.ticket.win {
		box-shadow:
			0 0 0 4px var(--gold),
			0 0 60px 6px rgba(255, 194, 61, 0.45),
			0 24px 60px -20px rgba(0, 0, 0, 0.8);
	}
	.head {
		text-align: center;
		margin-bottom: 10px;
	}
	.head h2 {
		font-size: 28px;
		color: var(--edge-ink);
		text-shadow: 0 2px 0 var(--edge-shadow);
	}
	.ticket :global(.rules) {
		text-align: center;
		font-size: 13px;
		font-weight: 600;
		opacity: 0.75;
		margin: -6px 0 10px;
	}
	.ticket.golden {
		background:
			radial-gradient(circle at 0 50%, transparent 10px, #ffe7a3 10.5px) left / 51% 100% no-repeat,
			radial-gradient(circle at 100% 50%, transparent 10px, #ffe7a3 10.5px) right / 51% 100%
				no-repeat;
		animation: gild 2.4s ease-in-out infinite alternate;
	}
	@keyframes gild {
		to {
			box-shadow:
				0 0 0 4px var(--edge),
				0 0 50px 4px rgba(255, 194, 61, 0.5),
				0 24px 60px -20px rgba(0, 0, 0, 0.8);
		}
	}
	.ticket :global(.play) {
		position: relative;
		aspect-ratio: 1 / 0.9;
		border-radius: 16px;
		background: #f6deae;
	}
	.ticket :global(.play.square) {
		aspect-ratio: 1;
	}
	.ticket :global(.play.wide) {
		aspect-ratio: 3 / 2;
	}
	/* The Cash Wheel's spot is a strip; the wheel sits under it */
	.ticket :global(.play.spot) {
		aspect-ratio: 2.4;
	}
	.ticket :global(.play.golden-play) {
		aspect-ratio: 1 / 0.8;
	}
	.outcome {
		min-height: 64px;
		text-align: center;
	}
	.title {
		font: 400 30px/1.1 var(--display);
		color: var(--muted);
		animation: bounce 0.6s cubic-bezier(0.3, 1.8, 0.5, 1) both;
	}
	.title.win {
		color: var(--gold);
		text-shadow: 0 3px 0 #9a5a00;
	}
	@keyframes bounce {
		from {
			opacity: 0;
			transform: scale(0.6);
		}
	}
	.sub,
	.hint {
		color: var(--muted);
		margin-top: 4px;
	}
	.err {
		color: var(--danger);
		font-weight: 600;
	}
	footer {
		width: min(420px, 100%);
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: auto;
	}
</style>
