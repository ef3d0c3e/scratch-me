<script lang="ts">
	import type { PageProps } from './$types';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { prizeText } from '#lib/prize.ts';

	let { data }: PageProps = $props();
	const i18n = getI18n();
	const earned = $derived(data.achievements.filter((a) => a.earned_at).length);
	const name = (id: string) =>
		i18n.has(`achievement.${id}`) ? i18n.t(`achievement.${id}` as never) : id;
	const how = (id: string) =>
		i18n.has(`achievement.${id}.how`) ? i18n.t(`achievement.${id}.how` as never) : '';
</script>

<svelte:head><title>{i18n.t('trophies.title')} · Scratch-me</title></svelte:head>

<div class="trophies">
	<div class="head">
		<h1>{i18n.t('trophies.title')}</h1>
		<p class="lead">{i18n.t('trophies.count', { n: earned, total: data.achievements.length })}</p>
	</div>
	<ul>
		{#each data.achievements as a (a.id)}
			<li class:earned={!!a.earned_at} data-achievement={a.id}>
				<span class="cup" aria-hidden="true">{a.earned_at ? '🏆' : '🔒'}</span>
				<div class="what">
					<h2>{name(a.id)}</h2>
					<p>{how(a.id)}</p>
					{#if a.progress && !a.earned_at}
						<div class="bar" aria-hidden="true">
							<span style:width="{(a.progress[0] / a.progress[1]) * 100}%"></span>
						</div>
						<p class="num small">{i18n.num(a.progress[0])} / {i18n.num(a.progress[1])}</p>
					{:else if a.earned_at}
						<p class="small">{i18n.date(a.earned_at)}</p>
					{/if}
				</div>
				<span class="reward">{prizeText(i18n, a.reward)}</span>
			</li>
		{/each}
	</ul>
</div>

<style>
	.trophies {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	h1 {
		font-size: 26px;
	}
	.lead {
		color: var(--muted);
		margin-top: 4px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	li {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		border-radius: 16px;
		background: var(--night-2);
		border: 1px solid var(--line);
		opacity: 0.8;
	}
	li.earned {
		opacity: 1;
		border-color: var(--gold);
	}
	.cup {
		font-size: 28px;
	}
	.what {
		flex: 1;
		min-width: 0;
	}
	h2 {
		font: 700 15px/1.2 var(--body);
	}
	.what p {
		font-size: 13px;
		color: var(--muted);
	}
	.small {
		font-size: 12px !important;
		margin-top: 2px;
	}
	.bar {
		height: 6px;
		border-radius: 99px;
		background: var(--night);
		margin-top: 6px;
		overflow: hidden;
	}
	.bar span {
		display: block;
		height: 100%;
		background: var(--gold);
	}
	.reward {
		font-weight: 800;
		font-size: 13px;
		color: var(--gold);
		text-align: right;
		max-width: 34%;
	}
</style>
