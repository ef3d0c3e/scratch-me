<script lang="ts">
	import { tick } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { post } from '#lib/api.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { achievementName, feedText } from '#lib/prize.ts';
	import type { AnnouncementDto, Shouted } from '#lib/types.ts';

	const i18n = getI18n();
	const live = getLive();
	const session = getSession();
	const MAX = 300;

	let body = $state('');
	let busy = $state(false);
	let error = $state('');
	let list = $state<HTMLElement>();
	let notice = $state('');
	let noticeBusy = $state(false);

	// New shouts: follow them down, unless the player scrolled up to read
	let stuck = true;
	$effect(() => {
		void live.messages.length;
		if (stuck) tick().then(() => list?.scrollTo({ top: list.scrollHeight }));
	});
	function onscroll() {
		if (list) stuck = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
	}

	async function send(e: SubmitEvent) {
		e.preventDefault();
		if (!body.trim()) return;
		busy = true;
		error = '';
		try {
			const r = await post<Shouted>('/shoutbox', { body });
			live.posted(r.shout);
			session.apply(r.progress);
			body = '';
			stuck = true;
		} catch (err) {
			error = i18n.error(err);
		} finally {
			busy = false;
		}
	}

	async function announce(e: SubmitEvent) {
		e.preventDefault();
		noticeBusy = true;
		try {
			await post<AnnouncementDto>('/announcements', { body: notice });
			notice = '';
		} catch (err) {
			error = i18n.error(err);
		} finally {
			noticeBusy = false;
		}
	}

	const time = (iso: string) =>
		new Date(iso).toLocaleTimeString(i18n.lang, { hour: '2-digit', minute: '2-digit' });
</script>

<svelte:head><title>{i18n.t('chat.title')} · Scratch-me</title></svelte:head>

<div class="chat">
	<div class="head">
		<h1>{i18n.t('chat.title')}</h1>
		<span
			class="dot"
			class:on={live.connected}
			title={i18n.t(live.connected ? 'chat.live' : 'chat.offline')}
			>{i18n.t(live.connected ? 'chat.live' : 'chat.offline')}</span
		>
	</div>

	{#if live.feed.length}
		<section class="feed" aria-label={i18n.t('feed.title')}>
			<ul>
				{#each live.feed.slice(0, 5) as item, i (item.at + item.user + i)}
					<li data-feed={item.type}>
						<span aria-hidden="true">{item.type === 'win' ? '💰' : '🏆'}</span>
						{feedText(i18n, item)}
					</li>
				{/each}
			</ul>
		</section>
	{/if}

	<section class="box" aria-label={i18n.t('chat.title')}>
		<ol bind:this={list} {onscroll}>
			{#each live.messages as m (m.id)}
				<li class:mine={m.user === session.me.username} data-shout={m.id}>
					<div class="who">
						<a href="/players/{encodeURIComponent(m.user)}">{m.user}</a>
						<span class="lvl">{m.level}</span>
						{#if m.title}<span class="title">{achievementName(i18n, m.title)}</span>{/if}
						<time datetime={m.at}>{time(m.at)}</time>
					</div>
					<p>{m.body}</p>
				</li>
			{:else}
				<li class="empty">{i18n.t('chat.empty')}</li>
			{/each}
		</ol>
		<form onsubmit={send}>
			<input
				name="shout"
				aria-label={i18n.t('chat.say')}
				placeholder={i18n.t('chat.say')}
				maxlength={MAX}
				autocomplete="off"
				bind:value={body}
			/>
			<button type="submit" class="btn primary small" disabled={busy || !body.trim()}
				>{i18n.t('chat.send')}</button
			>
		</form>
		{#if error}<p class="err" role="alert">{error}</p>{/if}
	</section>

	{#if session.me.is_admin}
		<form class="admin" onsubmit={announce}>
			<label>
				{i18n.t('chat.announce')}
				<textarea name="announcement" rows="2" maxlength="500" bind:value={notice}></textarea>
			</label>
			<button type="submit" class="btn small" disabled={noticeBusy || !notice.trim()}
				>{i18n.t('chat.announce_send')}</button
			>
		</form>
	{/if}
</div>

<style>
	.chat {
		display: flex;
		flex-direction: column;
		gap: 12px;
		flex: 1;
	}
	.head {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	h1 {
		font-size: 26px;
	}
	.dot {
		font-size: 12px;
		font-weight: 700;
		color: var(--muted);
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.dot::before {
		content: '';
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--muted);
	}
	.dot.on::before {
		background: var(--mint);
		box-shadow: 0 0 8px var(--mint);
	}
	.feed ul {
		list-style: none;
		margin: 0;
		padding: 10px 12px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--line);
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 13px;
	}
	.feed li:first-child {
		animation: slide 0.5s ease both;
	}
	@keyframes slide {
		from {
			opacity: 0;
			transform: translateY(-6px);
		}
	}
	.box {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		border-radius: 18px;
		background: var(--night-2);
		border: 1px solid var(--line);
		overflow: hidden;
	}
	ol {
		list-style: none;
		margin: 0;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		overflow-y: auto;
		max-height: 52dvh;
		min-height: 240px;
	}
	li.mine p {
		background: var(--night-3);
	}
	.who {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
	}
	.who a {
		font-weight: 800;
		color: var(--ink);
		text-decoration: none;
	}
	.lvl {
		min-width: 20px;
		height: 18px;
		padding: 0 5px;
		border-radius: 6px;
		display: grid;
		place-items: center;
		font: 400 11px/1 var(--display);
		color: var(--mint);
		background: var(--night);
	}
	.title {
		color: var(--gold);
		font-weight: 700;
	}
	time {
		margin-left: auto;
		color: var(--muted);
	}
	li p {
		margin-top: 3px;
		padding: 8px 10px;
		border-radius: 12px;
		background: var(--night);
		overflow-wrap: anywhere;
	}
	.empty {
		color: var(--muted);
		text-align: center;
		padding: 30px 0;
	}
	form {
		display: flex;
		gap: 8px;
		padding: 10px;
		border-top: 1px solid var(--line);
	}
	input,
	textarea {
		flex: 1;
		min-width: 0;
		border: 1.5px solid var(--line);
		background: var(--night);
		border-radius: 12px;
		padding: 10px 12px;
		font: inherit;
		color: inherit;
	}
	.err {
		color: var(--danger);
		font-size: 13px;
		font-weight: 600;
		padding: 0 12px 10px;
	}
	.admin {
		flex-direction: column;
		border: 1px dashed var(--gold);
		border-radius: 14px;
	}
	.admin label {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-weight: 700;
		font-size: 13px;
	}
</style>
