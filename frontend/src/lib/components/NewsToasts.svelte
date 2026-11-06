<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { getSession, type News } from '#lib/session.svelte.ts';
	import { prizeText } from '#lib/prize.ts';

	/**
	 * Level ups, quests done and achievements, as they happen: one at a time
	 * at the top of the screen, over everything. A level up gets the most room.
	 */
	const i18n = getI18n();
	const session = getSession();
	const shown = $derived(session.news[0] ?? null);

	// Each stays a few seconds, longer while a ticket's prize is still showing
	$effect(() => {
		const n = shown;
		if (!n) return;
		const t = setTimeout(() => session.dismiss(n.id), n.kind === 'level' ? 4200 : 3200);
		return () => clearTimeout(t);
	});

	function title(n: News) {
		switch (n.kind) {
			case 'level':
				return i18n.t('news.level', { n: Number(n.what) });
			case 'quest':
				return i18n.t('news.quest');
			case 'bonus':
				return i18n.t('news.bonus');
			case 'achievement':
				return i18n.t('news.achievement');
			case 'gift':
			case 'duel':
			case 'pot':
			case 'welcome':
				return n.what;
		}
	}

	function what(n: News) {
		if (n.kind === 'quest')
			return i18n.has(`quest.${n.what}`) ? i18n.t(`quest.${n.what}` as never) : n.what;
		if (n.kind === 'achievement')
			return i18n.has(`achievement.${n.what}`) ? i18n.t(`achievement.${n.what}` as never) : n.what;
		return '';
	}
	const ICONS = {
		level: '⭐',
		quest: '✅',
		bonus: '🎁',
		achievement: '🏆',
		gift: '🎁',
		duel: '⚔️',
		pot: '🍯',
		welcome: '🎉'
	};
</script>

<div class="news" aria-live="polite">
	{#if shown}
		{#key shown.id}
			<button
				type="button"
				class="toast {shown.kind}"
				onclick={() => session.dismiss(shown.id)}
				data-kind={shown.kind}
			>
				<span class="icon" aria-hidden="true">{ICONS[shown.kind]}</span>
				<span class="text">
					<b>{title(shown)}</b>
					{#if what(shown)}<span>{what(shown)}</span>{/if}
					<span class="reward">{prizeText(i18n, shown.reward)}</span>
				</span>
			</button>
		{/key}
	{/if}
</div>

<style>
	.news {
		position: fixed;
		top: calc(10px + var(--safe-top));
		left: 0;
		right: 0;
		z-index: 90;
		display: flex;
		justify-content: center;
		pointer-events: none;
		padding: 0 16px;
	}
	.toast {
		pointer-events: auto;
		width: min(440px, 100%);
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		border-radius: 18px;
		border: 1px solid var(--line);
		background: var(--night-2);
		color: var(--ink);
		text-align: left;
		box-shadow: 0 16px 40px -12px rgba(0, 0, 0, 0.7);
		animation: drop 0.5s cubic-bezier(0.3, 1.5, 0.5, 1) both;
	}
	@keyframes drop {
		from {
			transform: translateY(-120%);
			opacity: 0;
		}
	}
	.toast.level {
		background: linear-gradient(135deg, var(--gold), var(--gold-deep));
		color: #3a1d00;
		border: 0;
		padding: 16px 18px;
	}
	.toast.achievement {
		border-color: var(--gold);
	}
	.icon {
		font-size: 28px;
	}
	.text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		font-size: 14px;
	}
	.text b {
		font: 400 17px/1.1 var(--display);
	}
	.level .text b {
		font-size: 22px;
	}
	.reward {
		font-weight: 800;
		color: var(--gold);
	}
	.level .reward {
		color: #3a1d00;
	}
</style>
