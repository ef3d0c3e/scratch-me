<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { api, post } from '#lib/api.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import type { DuelDto } from '#lib/types.ts';

	/** Challenges to answer, ones waiting, and the last few played */
	const i18n = getI18n();
	const live = getLive();
	const session = getSession();
	let duels = $state<DuelDto[]>([]);
	let busy = $state<number | null>(null);
	let error = $state('');
	const me = $derived(session.me.username);

	async function load() {
		duels = await api<DuelDto[]>('/duels').catch(() => duels);
	}
	$effect(() => {
		void live.social;
		void load();
	});

	async function answer(d: DuelDto, action: 'accept' | 'decline' | 'cancel') {
		busy = d.id;
		error = '';
		try {
			await post<DuelDto>(`/duels/${d.id}`, { action });
			// The stake, and on accept the duel ticket
			await session.refresh();
			await load();
			if (action === 'accept') {
				const t = session.me.tickets.findLast((x) => x.series === 'duel');
				if (t) session.scratch(t);
			}
		} catch (err) {
			error = i18n.error(err);
			await load();
		} finally {
			busy = null;
		}
	}
	const other = (d: DuelDto) => (d.challenger === me ? d.opponent : d.challenger);
</script>

{#if duels.length}
	<section class="duels" aria-labelledby="duels-title">
		<h2 id="duels-title">⚔️ {i18n.t('duel.title')}</h2>
		<ul>
			{#each duels.slice(0, 5) as d (d.id)}
				<li data-duel={d.id} data-status={d.status}>
					{#if d.status === 'open' && d.opponent === me}
						<span>{i18n.t('duel.challenges_you', { name: d.challenger, n: d.stake })}</span>
						<span class="acts">
							<button
								type="button"
								class="btn small primary"
								disabled={busy !== null || session.me.coins < d.stake}
								onclick={() => answer(d, 'accept')}>{i18n.t('duel.accept')}</button
							>
							<button
								type="button"
								class="btn small ghost"
								disabled={busy !== null}
								onclick={() => answer(d, 'decline')}>{i18n.t('duel.decline')}</button
							>
						</span>
					{:else if d.status === 'open'}
						<span>{i18n.t('duel.waiting', { name: d.opponent })}</span>
						<button
							type="button"
							class="btn small ghost"
							disabled={busy !== null}
							onclick={() => answer(d, 'cancel')}>{i18n.t('duel.cancel')}</button
						>
					{:else if d.status === 'accepted'}
						<span>{i18n.t('duel.vs', { name: other(d) })}</span>
						<span class="done" class:won={d.winner === me}
							>{i18n.t('duel.won_by', {
								name: d.winner === me ? i18n.t('duel.you') : (d.winner ?? '')
							})}</span
						>
					{:else}
						<span>{i18n.t('duel.vs', { name: other(d) })}</span>
						<span class="done">{i18n.t(`duel.status.${d.status}`)}</span>
					{/if}
				</li>
			{/each}
		</ul>
		{#if error}<p class="err" role="alert">{error}</p>{/if}
	</section>
{/if}

<style>
	.duels {
		padding: 14px;
		border-radius: 18px;
		background: var(--night-2);
		border: 1px solid var(--line);
	}
	h2 {
		font-size: 15px;
		margin-bottom: 8px;
	}
	ul {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		font-size: 14px;
	}
	li > span:first-child {
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.acts {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}
	.done {
		font-size: 12px;
		opacity: 0.8;
		flex-shrink: 0;
	}
	.done.won {
		color: #facc15;
		opacity: 1;
	}
</style>
