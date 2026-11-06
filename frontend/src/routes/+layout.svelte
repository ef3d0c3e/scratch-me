<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import type { LayoutProps } from './$types';
	import { setI18n } from '#lib/i18n/index.svelte.ts';
	import { setFx } from '#lib/fx/fx.svelte.ts';

	let { data, children }: LayoutProps = $props();
	// svelte-ignore state_referenced_locally
	setI18n(data.lang);
	const fx = setFx();
	// Mounted last, after every child: browser tests wait for this before clicking
	onMount(() => {
		fx.load();
		document.documentElement.dataset.hydrated = '';
	});
</script>

<svelte:head>
	<title>Scratch-me</title>
</svelte:head>

{@render children()}
