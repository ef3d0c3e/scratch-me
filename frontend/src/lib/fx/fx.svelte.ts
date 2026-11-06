/**
 * Sound and vibration. One per visitor, through context. The audio context
 * starts on the first touch (browsers refuse it before), and the player's
 * choices are kept in this browser only.
 */

import { getContext, setContext } from 'svelte';
import type { Tier } from './celebrate.ts';
import { melody, scratchVoice, tickDue, vibration } from './sound.ts';

const STORE = 'scratchme:fx';

export class Fx {
	sound = $state(true);
	haptics = $state(true);

	private ctx: AudioContext | null = null;
	private master: GainNode | null = null;
	private noise: { src: AudioBufferSourceNode; filter: BiquadFilterNode; gain: GainNode } | null =
		null;
	private lastTick = 0;

	/** Reads the saved choices; call once in the browser */
	load() {
		try {
			const saved = JSON.parse(localStorage.getItem(STORE) ?? '{}');
			if (typeof saved.sound === 'boolean') this.sound = saved.sound;
			if (typeof saved.haptics === 'boolean') this.haptics = saved.haptics;
		} catch {
			// Storage off (private mode): the defaults stand
		}
	}

	private save() {
		try {
			localStorage.setItem(STORE, JSON.stringify({ sound: this.sound, haptics: this.haptics }));
		} catch {
			// Not kept, still applies for this visit
		}
	}

	toggleSound() {
		this.sound = !this.sound;
		if (!this.sound) this.scratchStop();
		this.save();
	}

	toggleHaptics() {
		this.haptics = !this.haptics;
		this.save();
	}

	/** Starts audio; must run inside a tap or touch */
	unlock() {
		if (!this.sound) return;
		try {
			if (!this.ctx) {
				const Ctx =
					window.AudioContext ??
					(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
				if (!Ctx) return;
				this.ctx = new Ctx();
				this.master = this.ctx.createGain();
				this.master.gain.value = 0.9;
				this.master.connect(this.ctx.destination);
			}
			if (this.ctx.state === 'suspended') void this.ctx.resume();
		} catch {
			this.ctx = null;
		}
	}

	/** The scratch noise follows the finger; `speed` in px/ms, 0 when it stops */
	scratch(speed: number, pitch = 1, q = 0.9) {
		if (this.haptics && tickDue(performance.now(), this.lastTick, speed)) {
			this.lastTick = performance.now();
			navigator.vibrate?.(4);
		}
		if (!this.sound || !this.ctx || !this.master) return;
		if (!this.noise) this.noise = this.startNoise(this.ctx, this.master);
		const v = scratchVoice(speed);
		const t = this.ctx.currentTime;
		this.noise.gain.gain.setTargetAtTime(v.gain, t, 0.03);
		this.noise.filter.frequency.setTargetAtTime(v.freq * pitch, t, 0.05);
		this.noise.filter.Q.setTargetAtTime(q, t, 0.05);
	}

	scratchStop() {
		if (this.noise && this.ctx) this.noise.gain.gain.setTargetAtTime(0, this.ctx.currentTime, 0.04);
	}

	/** A click, as a wheel segment passes the pointer */
	tick() {
		if (!this.sound || !this.ctx || !this.master) return;
		const t = this.ctx.currentTime;
		const osc = this.ctx.createOscillator();
		const env = this.ctx.createGain();
		osc.type = 'square';
		osc.frequency.value = 1800;
		env.gain.setValueAtTime(0.06, t);
		env.gain.exponentialRampToValueAtTime(0.0001, t + 0.03);
		osc.connect(env).connect(this.master);
		osc.start(t);
		osc.stop(t + 0.04);
	}

	/** The prize's jingle and buzz */
	win(tier: Tier) {
		if (this.haptics) {
			const pattern = vibration(tier);
			if (pattern.length) navigator.vibrate?.(pattern);
		}
		if (!this.sound || !this.ctx || !this.master) return;
		const t0 = this.ctx.currentTime + 0.02;
		for (const note of melody(tier)) {
			const osc = this.ctx.createOscillator();
			const env = this.ctx.createGain();
			osc.type = note.wave;
			osc.frequency.value = note.f;
			const start = t0 + note.at;
			env.gain.setValueAtTime(0, start);
			env.gain.linearRampToValueAtTime(note.vol, start + 0.012);
			env.gain.exponentialRampToValueAtTime(0.0001, start + note.len);
			osc.connect(env).connect(this.master);
			osc.start(start);
			osc.stop(start + note.len + 0.05);
		}
	}

	/** Brown-ish noise through a band-pass: foil under a coin */
	private startNoise(ctx: AudioContext, out: AudioNode) {
		const buffer = ctx.createBuffer(1, ctx.sampleRate * 2, ctx.sampleRate);
		const data = buffer.getChannelData(0);
		let last = 0;
		for (let i = 0; i < data.length; i++) {
			const white = Math.random() * 2 - 1;
			// Half white, half smoothed: grainy rather than hissy
			last = (last + 0.08 * white) / 1.08;
			data[i] = 0.5 * white + 2.5 * last;
		}
		const src = ctx.createBufferSource();
		src.buffer = buffer;
		src.loop = true;
		const filter = ctx.createBiquadFilter();
		filter.type = 'bandpass';
		filter.Q.value = 0.9;
		const gain = ctx.createGain();
		gain.gain.value = 0;
		src.connect(filter).connect(gain).connect(out);
		src.start();
		return { src, filter, gain };
	}
}

const KEY = Symbol('fx');
export const setFx = () => setContext(KEY, new Fx());
export const getFx = () => getContext<Fx>(KEY);
