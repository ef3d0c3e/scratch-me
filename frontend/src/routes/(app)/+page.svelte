<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import { countdown, due } from '#lib/time.ts';
	import type { EnvelopeClaim, PocketClaim } from '#lib/types.ts';
	import { ticketsText } from '#lib/prize.ts';
	import QuestsCard from '#lib/components/QuestsCard.svelte';
	import PotCard from '#lib/components/PotCard.svelte';
	import DuelsCard from '#lib/components/DuelsCard.svelte';
	import CharmsWorn from '#lib/components/CharmsWorn.svelte';
	import SealBadge from '#lib/components/SealBadge.svelte';
	import { paperOf } from '#lib/seals.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { feedText } from '#lib/prize.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { getFx } from '#lib/fx/fx.svelte.ts';
	import { SERIES_LOOK } from '#lib/series.ts';

	const i18n = getI18n();
	const fx = getFx();

	const session = getSession();
	const live = getLive();
	/** Announcements this browser closed (kept per browser, best effort) */
	let dismissed = $state<number[]>([]);
	onMount(() => {
		try {
			dismissed = JSON.parse(localStorage.getItem('scratchme:dismissed') ?? '[]');
		} catch {
			dismissed = [];
		}
	});
	function dismiss(id: number) {
		dismissed = [...dismissed, id];
		try {
			localStorage.setItem('scratchme:dismissed', JSON.stringify(dismissed.slice(-50)));
		} catch {
			// Not kept: it comes back next visit
		}
	}
	const notice = $derived(live.announcements.find((a) => !dismissed.includes(a.id)));
	const me = $derived(session.me);
	let now = $state(Date.now());
	let busy = $state<'pocket' | 'envelope' | null>(null);
	let error = $state('');
	/** A short "+3 tickets" over the card that gave them */
	let gain = $state<{ where: 'pocket' | 'envelope'; text: string; key: number } | null>(null);

	onMount(() => {
		const id = setInterval(() => {
			now = Date.now();
			// A timer ran out: the pocket or the envelope has something new
			if (due(me.pocket.next_at, now) || (!me.envelope.available && due(me.envelope.next_at, now)))
				refresh();
		}, 1000);
		return () => clearInterval(id);
	});

	let refreshing = false;
	async function refresh() {
		if (refreshing) return;
		refreshing = true;
		try {
			await session.refresh();
		} catch (e) {
			error = i18n.error(e);
		} finally {
			refreshing = false;
		}
	}

	function flash(where: 'pocket' | 'envelope', parts: string[]) {
		gain = { where, text: parts.join(' · '), key: Date.now() };
		setTimeout(() => {
			if (gain && Date.now() - gain.key >= 1400) gain = null;
		}, 1500);
	}

	async function takePocket() {
		busy = 'pocket';
		error = '';
		try {
			const r = await post<PocketClaim>('/pocket/claim');
			session.add(r.tickets);
			session.me.pocket = r.pocket;
			session.apply(r.progress);
			flash('pocket', [i18n.t('lobby.got_tickets', { n: r.tickets.length })]);
		} catch (e) {
			error = i18n.error(e);
			await refresh();
		} finally {
			busy = null;
		}
	}

	async function openEnvelope() {
		busy = 'envelope';
		error = '';
		try {
			const r = await post<EnvelopeClaim>('/envelope/claim');
			session.add(r.tickets);
			session.apply(r.progress);
			const parts = [];
			if (r.contents.coins) parts.push(i18n.t('lobby.got_coins', { n: r.contents.coins }));
			if (r.tickets.length) parts.push(i18n.t('lobby.got_tickets', { n: r.tickets.length }));
			flash('envelope', parts);
			await refresh();
		} catch (e) {
			error = i18n.error(e);
			await refresh();
		} finally {
			busy = null;
		}
	}

	const next = () => session.scratch();

	async function logout() {
		await post('/auth/logout').catch(() => {});
		location.assign('/login');
	}

	const stack = $derived(me.tickets.slice(0, 3));
	const env = $derived(me.envelope);

	/** Days of this week's cycle already opened, and the one waiting today */
	const week = $derived(
		env.available
			? { done: (env.day - 1) % 7, today: (env.day - 1) % 7 }
			: { done: ((env.streak - 1) % 7) + 1, today: -1 }
	);
</script>

<svelte:head><title>Scratch-me</title></svelte:head>

<div class="lobby">
	{#if notice}
		<aside class="notice" aria-label={i18n.t('lobby.announcement')}>
			<span aria-hidden="true">📣</span>
			<p>{notice.body}</p>
			<button type="button" onclick={() => dismiss(notice.id)} aria-label={i18n.t('scratch.close')}
				>✕</button
			>
		</aside>
	{/if}
	{#if live.feed[0]}
		{#key live.feed[0].at + live.feed[0].user}
			<p class="ticker" data-testid="ticker">
				<span aria-hidden="true">{live.feed[0].type === 'win' ? '💰' : '🏆'}</span>
				{feedText(i18n, live.feed[0])}
			</p>
		{/key}
	{/if}
	{#if me.guest}
		<aside class="guest" aria-labelledby="guest-title">
			<div>
				<h2 id="guest-title">{i18n.t('guest.title')}</h2>
				<p>{i18n.t('guest.lead')}</p>
			</div>
			<a class="btn small primary" href="/keep">{i18n.t('guest.keep')}</a>
		</aside>
	{/if}
	<section class="stack" aria-labelledby="stack-title">
		{#if stack.length}
			<!-- A big tap target for the same action as the button below -->
			<button type="button" class="pile" onclick={next} tabindex="-1" aria-hidden="true">
				{#each stack.toReversed() as t, i (t.id)}
					<span
						class="stub"
						style:--k={stack.length - 1 - i}
						style:--edge={SERIES_LOOK[t.series].edge}
						style:--edge-ink={SERIES_LOOK[t.series].ink}
						style:--paper-2={paperOf(t)}
						data-seal={t.seal?.key}
					>
						<span class="stub-name">{i18n.t(`series.${t.series}`)}</span>
						{#if t.seal}<span class="stub-seal"><SealBadge seal={t.seal} small /></span>{/if}
						<span class="stub-foil"></span>
						<span class="stub-src"
							>{t.from ? i18n.t('stub.from', { name: t.from }) : i18n.t(`source.${t.source}`)}</span
						>
					</span>
				{/each}
			</button>
			<h1 id="stack-title" class="count">{i18n.t('lobby.stack', { n: me.tickets.length })}</h1>
			<button type="button" class="btn primary cta" onclick={next}
				>{i18n.t('lobby.scratch_next')}</button
			>
		{:else}
			<div class="empty">
				<h1 id="stack-title" class="count">{i18n.t('lobby.stack_empty')}</h1>
				<p class="hint">{i18n.t('lobby.stack_empty_hint')}</p>
			</div>
		{/if}
	</section>

	{#if error}<p class="err" role="alert">{error}</p>{/if}

	<CharmsWorn />

	<section class="drops">
		<article class="drop pocket" class:ready={me.pocket.count > 0}>
			<h2>{i18n.t('lobby.pocket')}</h2>
			<div class="slots" aria-hidden="true">
				{#each { length: me.pocket.size }, i (i)}
					<span class="slot" class:full={i < me.pocket.count}></span>
				{/each}
			</div>
			<p class="worth">{i18n.t('lobby.pocket_worth', { n: me.pocket_worth })}</p>
			<p class="state">
				{#if me.pocket.count > 0}
					{i18n.t('lobby.pocket_ready', { n: me.pocket.count })}
				{/if}
				{#if me.pocket.next_at}
					<span class="timer num"
						>{i18n.t('lobby.pocket_next', { time: countdown(me.pocket.next_at, now) })}</span
					>
				{:else}
					<span class="timer">{i18n.t('lobby.pocket_full')}</span>
				{/if}
			</p>
			<button
				type="button"
				class="btn small"
				class:primary={me.pocket.count > 0}
				disabled={me.pocket.count === 0 || busy !== null}
				onclick={takePocket}>{i18n.t('lobby.pocket_take')}</button
			>
			{#if gain?.where === 'pocket'}{#key gain.key}<span class="gain">{gain.text}</span>{/key}{/if}
		</article>

		<article class="drop envelope" class:ready={env.available}>
			<h2>{i18n.t('lobby.envelope')}</h2>
			<p class="streak">
				<span aria-hidden="true">🔥</span>
				{i18n.t('lobby.streak', { n: env.streak })}
			</p>
			<div class="week" aria-hidden="true">
				{#each { length: 7 }, i (i)}
					<span
						class="day"
						class:done={i < week.done}
						class:today={i === week.today}
						class:big={i === 6}>{i + 1}</span
					>
				{/each}
			</div>
			<p class="state">
				{#if env.available}
					{i18n.t('lobby.envelope_ready', { n: env.day })}
				{:else}
					<span class="timer num"
						>{i18n.t('lobby.envelope_next', { time: countdown(env.next_at, now) })}</span
					>
				{/if}
			</p>
			<p class="holds">
				{i18n.t('lobby.envelope_holds')}:
				{[
					env.contents.coins ? i18n.t('contents.coins', { n: env.contents.coins }) : '',
					ticketsText(i18n, env.contents.tickets)
				]
					.filter(Boolean)
					.join(' + ')}
			</p>
			<button
				type="button"
				class="btn small"
				class:primary={env.available}
				disabled={!env.available || busy !== null}
				onclick={openEnvelope}>{i18n.t('lobby.envelope_open')}</button
			>
			{#if gain?.where === 'envelope'}{#key gain.key}<span class="gain">{gain.text}</span
					>{/key}{/if}
		</article>
	</section>

	{#if !me.guest}<DuelsCard />{/if}
	<PotCard {now} />
	<QuestsCard />

	<footer class="foot">
		<a href="/players/{encodeURIComponent(me.username)}">{me.username}</a>
		<button
			type="button"
			class="toggle"
			aria-pressed={fx.sound}
			onclick={() => {
				fx.toggleSound();
				fx.unlock();
			}}>{fx.sound ? '🔊' : '🔇'} {i18n.t('settings.sound')}</button
		>
		<button
			type="button"
			class="toggle"
			aria-pressed={fx.haptics}
			onclick={() => fx.toggleHaptics()}>📳 {i18n.t('settings.haptics')}</button
		>
		{#if !me.guest}
			<!-- A guest logging out could never come back: they sign up instead -->
			<button type="button" class="link" onclick={logout}>{i18n.t('auth.logout')}</button>
		{/if}
	</footer>
</div>

<style>
	.worth {
		font-size: 12px;
		color: var(--muted);
	}
	.guest {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 14px;
		border-radius: 16px;
		background: linear-gradient(135deg, rgba(87, 184, 255, 0.22), var(--night-2));
		border: 1px solid rgba(87, 184, 255, 0.5);
	}
	.guest h2 {
		font-size: 15px;
	}
	.guest p {
		font-size: 13px;
		opacity: 0.85;
	}
	.guest .btn {
		flex-shrink: 0;
	}
	.notice {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px 14px;
		border-radius: 16px;
		background: linear-gradient(135deg, var(--pink), #b8336a);
		color: #fff;
		font-weight: 600;
	}
	.notice p {
		flex: 1;
		overflow-wrap: anywhere;
	}
	.notice button {
		border: 0;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 50%;
		width: 28px;
		height: 28px;
		color: #fff;
	}
	.ticker {
		margin: -8px 0;
		font-size: 13px;
		color: var(--muted);
		text-align: center;
		animation: tick-in 0.5s ease both;
	}
	@keyframes tick-in {
		from {
			opacity: 0;
			transform: translateY(-6px);
		}
	}
	.lobby {
		display: flex;
		flex-direction: column;
		gap: 20px;
		flex: 1;
	}

	/* ---------- the stack ---------- */
	.stack {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding-top: 8px;
	}
	.pile {
		position: relative;
		width: min(300px, 80vw);
		aspect-ratio: 16 / 10;
		border: 0;
		padding: 0;
		background: none;
	}
	.stub {
		position: absolute;
		inset: 0;
		display: grid;
		grid-template-rows: auto 1fr auto;
		gap: 8px;
		padding: 12px 18px;
		border-radius: 14px;
		color: var(--paper-ink);
		background:
			radial-gradient(circle at 0 50%, transparent 9px, var(--paper-2) 9.5px) left / 51% 100%
				no-repeat,
			radial-gradient(circle at 100% 50%, transparent 9px, var(--paper-2) 9.5px) right / 51% 100%
				no-repeat;
		box-shadow:
			0 0 0 3px var(--edge),
			0 14px 30px -12px rgba(0, 0, 0, 0.7);
		transform: translateY(calc(var(--k) * -10px)) rotate(calc(var(--k) * 4deg - 4deg))
			scale(calc(1 - var(--k) * 0.04));
		transition: transform 0.25s ease;
	}
	.pile:hover .stub:last-child,
	.pile:focus-visible .stub:last-child {
		transform: translateY(-6px) rotate(-2deg);
	}
	.stub-seal {
		position: absolute;
		top: 6px;
		right: 10px;
	}
	.stub-name {
		font: 400 18px/1 var(--display);
		color: var(--edge-ink);
		text-align: left;
	}
	.stub-foil {
		border-radius: 10px;
		background:
			linear-gradient(115deg, transparent 35%, rgba(255, 255, 255, 0.6) 50%, transparent 65%) 0 0 /
				250% 100%,
			linear-gradient(135deg, #d9d6e6, #9d98b5);
		animation: shine 3s ease-in-out infinite;
	}
	@keyframes shine {
		0%,
		55% {
			background-position:
				120% 0,
				0 0;
		}
		100% {
			background-position:
				-120% 0,
				0 0;
		}
	}
	.stub-src {
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		text-align: right;
		opacity: 0.6;
	}
	.count {
		font: 700 18px/1.3 var(--body);
	}
	.cta {
		width: min(300px, 80vw);
		font-size: 18px;
		min-height: 56px;
	}
	.empty {
		text-align: center;
		padding: 36px 16px;
		border: 2px dashed var(--line);
		border-radius: 18px;
		width: 100%;
	}
	.hint {
		color: var(--muted);
		margin-top: 4px;
	}
	.err {
		color: var(--danger);
		font-weight: 600;
		text-align: center;
	}

	/* ---------- free drops ---------- */
	.drops {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	.drop {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px;
		border-radius: 18px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	.drop.ready {
		border-color: rgba(255, 194, 61, 0.6);
		box-shadow: 0 0 24px -6px rgba(255, 194, 61, 0.45);
	}
	.drop h2 {
		font-size: 14px;
		letter-spacing: 0.02em;
	}
	.drop .btn {
		margin-top: auto;
	}
	.slots {
		display: flex;
		gap: 6px;
	}
	.slot {
		flex: 1;
		height: 22px;
		border-radius: 6px;
		border: 1.5px dashed var(--line);
	}
	.slot.full {
		border: 0;
		background: linear-gradient(135deg, var(--paper-2), #f3c46b);
		box-shadow: inset 0 0 0 2px var(--pink);
		animation: drop-in 0.4s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes drop-in {
		from {
			transform: translateY(-10px);
			opacity: 0;
		}
	}
	.state {
		font-size: 13px;
		font-weight: 600;
		display: flex;
		flex-direction: column;
	}
	.timer {
		color: var(--muted);
		font-weight: 500;
	}
	.streak {
		font-size: 13px;
		font-weight: 700;
		color: var(--gold);
	}
	.week {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 3px;
	}
	.day {
		display: grid;
		place-items: center;
		height: 20px;
		border-radius: 5px;
		font-size: 10.5px;
		font-weight: 700;
		color: var(--muted);
		background: var(--night);
	}
	.day.big {
		color: var(--gold);
	}
	.day.done {
		background: var(--mint);
		color: #063d29;
	}
	.day.today {
		background: var(--gold);
		color: #3a1d00;
		animation: pulse 1.4s ease-in-out infinite;
	}
	@keyframes pulse {
		50% {
			transform: scale(1.15);
		}
	}
	.holds {
		font-size: 12px;
		color: var(--muted);
	}
	.gain {
		position: absolute;
		left: 50%;
		top: 40%;
		transform: translateX(-50%);
		white-space: nowrap;
		font: 400 18px/1 var(--display);
		color: var(--gold);
		text-shadow: 0 2px 0 #9a5a00;
		pointer-events: none;
		animation: float-up 1.4s ease-out both;
	}
	@keyframes float-up {
		0% {
			opacity: 0;
			transform: translate(-50%, 10px) scale(0.8);
		}
		20% {
			opacity: 1;
			transform: translate(-50%, 0) scale(1.1);
		}
		100% {
			opacity: 0;
			transform: translate(-50%, -50px);
		}
	}
	.foot {
		margin-top: auto;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
		gap: 12px;
		color: var(--muted);
		font-size: 14px;
	}
	.toggle {
		border: 1px solid var(--line);
		background: none;
		border-radius: 999px;
		padding: 4px 10px;
		font-size: 13px;
		color: var(--muted);
	}
	.toggle[aria-pressed='true'] {
		color: var(--ink);
		border-color: var(--gold);
	}
	.link {
		border: 0;
		background: none;
		padding: 0;
		color: var(--gold);
		text-decoration: underline;
	}
</style>
