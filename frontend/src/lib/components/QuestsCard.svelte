<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { api, post } from '#lib/api.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { prizeText, questName } from '#lib/prize.ts';
	import type { QuestsDto } from '#lib/types.ts';
	import { countdown, due } from '#lib/time.ts';

	/** The round's three quests, the bonus for all three, and the hour's one swap */
	const i18n = getI18n();
	const session = getSession();
	let quests = $state<QuestsDto | null>(null);
	let busy = $state(false);
	let error = $state('');

	async function load() {
		try {
			quests = await api<QuestsDto>('/quests');
		} catch (e) {
			error = i18n.error(e);
		}
	}
	onMount(load);
	/** The hour turns: new quests */
	let now = $state(Date.now());
	onMount(() => {
		const id = setInterval(() => {
			now = Date.now();
			if (quests && due(quests.resets_at, now)) {
				quests = { ...quests, resets_at: new Date(now + 60_000).toISOString() };
				load();
			}
		}, 1000);
		return () => clearInterval(id);
	});
	// Progress landed somewhere: the bars move
	$effect(() => {
		if (session.progressed > 0) load();
	});

	async function reroll(key: string) {
		busy = true;
		error = '';
		try {
			quests = await post<QuestsDto>('/quests/reroll', { key });
		} catch (e) {
			error = i18n.error(e);
		} finally {
			busy = false;
		}
	}
	const name = (key: string, goal: number) => questName(i18n, key, goal);
	/** The last round done: nothing more today */
	const over = $derived(!!quests && quests.bonus_done && quests.round >= quests.rounds);
	const done = $derived(quests?.items.filter((q) => q.done).length ?? 0);
</script>

<section class="quests" aria-labelledby="quests-title">
	<div class="head">
		<h2 id="quests-title">{i18n.t('quests.title')}</h2>
		{#if quests}<span class="count">{done}/{quests.items.length}</span>{/if}
	</div>
	{#if quests}
		<p class="round" data-round={quests.round}>
			{i18n.t('quests.round', { n: quests.round, of: quests.rounds })}
			<span class="resets num" data-testid="quests-reset"
				>{i18n.t('quests.resets', { time: countdown(quests.resets_at, now) })}</span
			>
			<span class="pips" aria-hidden="true">
				{#each { length: quests.rounds }, i (i)}
					<span
						class="pip"
						class:past={i + 1 < quests.round || over}
						class:now={i + 1 === quests.round && !over}
					></span>
				{/each}
			</span>
		</p>
	{/if}
	{#if quests}
		<ul>
			{#each quests.items as q (`${quests.round}:${q.key}`)}
				<li class:done={q.done} data-quest={q.key}>
					<div class="row">
						<span class="name">{q.done ? '✅ ' : ''}{name(q.key, q.goal)}</span>
						<span class="reward">{prizeText(i18n, q.reward)}</span>
					</div>
					<div class="bar" aria-hidden="true">
						<span style:width="{(Math.min(q.progress, q.goal) / q.goal) * 100}%"></span>
					</div>
					<div class="row small">
						<span class="num">{i18n.num(Math.min(q.progress, q.goal))} / {i18n.num(q.goal)}</span>
						{#if !q.done && !quests.rerolled}
							<button
								type="button"
								class="reroll"
								disabled={busy}
								onclick={() => reroll(q.key)}
								aria-label="{i18n.t('quests.reroll')}: {name(q.key, q.goal)}"
								>🎲 {i18n.t('quests.reroll')}</button
							>
						{/if}
					</div>
				</li>
			{/each}
		</ul>
		<p class="bonus" class:got={quests.bonus_done}>
			🎁 {quests.bonus_done ? i18n.t('quests.bonus_done') : i18n.t('quests.bonus')}
			<b>{prizeText(i18n, quests.bonus)}</b>
			{#if over}
				<span class="then">{i18n.t('quests.all_done')}</span>
			{:else if quests.round < quests.rounds}
				<span class="then">{i18n.t('quests.next')}</span>
			{/if}
		</p>
	{/if}
	{#if error}<p class="err" role="alert">{error}</p>{/if}
</section>

<style>
	.quests {
		padding: 14px;
		border-radius: 18px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	.head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		margin-bottom: 10px;
	}
	h2 {
		font-size: 14px;
	}
	.count {
		font-weight: 800;
		color: var(--mint);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		font-weight: 600;
		font-size: 14px;
	}
	.row.small {
		font-size: 12px;
		color: var(--muted);
		font-weight: 500;
		min-height: 24px;
	}
	.reward {
		color: var(--gold);
		font-weight: 800;
		font-size: 13px;
		text-align: right;
	}
	.bar {
		height: 8px;
		border-radius: 99px;
		background: var(--night);
		margin: 6px 0 4px;
		overflow: hidden;
	}
	.bar span {
		display: block;
		height: 100%;
		border-radius: 99px;
		background: linear-gradient(90deg, var(--mint), #a6ffd9);
		transition: width 0.6s cubic-bezier(0.3, 1.3, 0.5, 1);
	}
	.done .name {
		color: var(--mint);
	}
	.reroll {
		border: 1px solid var(--line);
		background: none;
		border-radius: 999px;
		padding: 2px 10px;
		font-size: 12px;
		color: var(--muted);
	}
	.bonus {
		margin-top: 12px;
		padding-top: 10px;
		border-top: 1px dashed var(--line);
		font-size: 13px;
		color: var(--muted);
	}
	.bonus b {
		color: var(--gold);
	}
	.then {
		display: block;
		margin-top: 2px;
		font-size: 12px;
	}
	.round {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 12px;
		color: var(--muted);
		margin: -4px 0 10px;
	}
	.resets {
		margin-left: auto;
		margin-right: 8px;
	}
	.pips {
		display: flex;
		gap: 4px;
	}
	.pip {
		width: 18px;
		height: 6px;
		border-radius: 99px;
		background: var(--night);
		border: 1px solid var(--line);
	}
	.pip.past {
		background: var(--mint);
		border-color: var(--mint);
	}
	.pip.now {
		border-color: var(--mint);
	}
	.bonus.got {
		color: var(--mint);
	}
	.err {
		color: var(--danger);
		font-size: 13px;
	}
</style>
