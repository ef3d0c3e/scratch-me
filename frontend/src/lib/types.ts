/** Shapes of the API's responses, see backend/src/routes */

export type Series =
	| 'lucky_three'
	| 'high_card'
	| 'lucky_reels'
	| 'bingo'
	| 'number_match'
	| 'cash_wheel'
	| 'treasure_dig'
	| 'ticket_chain'
	| 'golden';
export type Source = 'pocket' | 'envelope' | 'prize' | 'shop' | 'reward';

export type ThemeId = 'midnight' | 'sunset' | 'mint' | 'neon' | 'royal';
export type ToolId = 'coin' | 'key' | 'eraser' | 'laser';
export type Cosmetic = { kind: 'theme'; id: ThemeId } | { kind: 'tool'; id: ToolId };

export interface TicketDto {
	id: number;
	series: Series;
	source: Source;
	created_at: string;
}

export interface Pocket {
	count: number;
	size: number;
	/** When the next ticket comes, null while full */
	next_at: string | null;
}

export interface Contents {
	coins: number;
	tickets: Series[];
}

export interface EnvelopeDto {
	available: boolean;
	streak: number;
	best: number;
	day: number;
	contents: Contents;
	next_at: string;
}

export interface MeDto {
	username: string;
	coins: number;
	pocket: Pocket;
	envelope: EnvelopeDto;
	theme: ThemeId;
	tool: ToolId;
	/** Cosmetics owned besides the defaults */
	owned: Cosmetic[];
	level: LevelDto;
	/** The achievement shown next to their name */
	title: string | null;
	is_admin: boolean;
	tickets: TicketDto[];
}

export interface LevelDto {
	level: number;
	xp: number;
	/** Where this level started and the next begins */
	from: number;
	to: number;
}

export interface Reward {
	/** A level reached, a quest key or an achievement id */
	what: string;
	reward: Prize;
}

/** What an action's progress gave */
export interface Progress {
	xp_gained: number;
	level: LevelDto;
	level_ups: Reward[];
	quests: Reward[];
	bonus: Prize | null;
	achievements: Reward[];
	tickets: TicketDto[];
	coins: number;
}

export interface QuestDto {
	key: string;
	goal: number;
	progress: number;
	done: boolean;
	reward: Prize;
}

export interface QuestsDto {
	items: QuestDto[];
	bonus: Prize;
	bonus_done: boolean;
	rerolled: boolean;
	resets_at: string;
}

export interface AchievementDto {
	id: string;
	reward: Prize;
	earned_at: string | null;
	progress: [number, number] | null;
}

export type Prize =
	| { kind: 'nothing' }
	| { kind: 'coins'; amount: number }
	| { kind: 'ticket'; series: Series }
	| { kind: 'tickets'; series: Series[] }
	| { kind: 'pocket_refill' }
	| { kind: 'cosmetic'; item: Cosmetic };

export type LuckySymbol = 'gem' | 'coin' | 'clover' | 'ticket' | 'hourglass' | 'star';

export interface LuckyLayout {
	cells: LuckySymbol[];
	win: LuckySymbol | null;
}

export interface NumberLayout {
	winning: number[];
	spots: { n: number; coins: number; hit: boolean }[];
}

export interface ChainLayout {
	panels: (Series | null)[];
}

export interface WheelLayout {
	base: number;
	wheel: number[];
	/** The segment the wheel stops on */
	stop: number;
}

export interface GoldenLayout {
	prize: Prize;
}

export interface Scratched {
	id: number;
	series: Series;
	layout: unknown;
	prize: Prize;
	coins: number;
	/** Tickets won */
	tickets: TicketDto[];
	/** The cosmetic won was already owned: coins instead */
	duplicate: boolean;
	progress: Progress;
}

export interface ShopDto {
	tickets: { series: Series; price: number; level: number }[];
	cosmetics: { item: Cosmetic; price: number }[];
}

export interface Bought {
	coins: number;
	ticket: TicketDto | null;
	item: Cosmetic | null;
	progress: Progress;
}

export interface PocketClaim {
	tickets: TicketDto[];
	pocket: Pocket;
	progress: Progress;
}

export interface EnvelopeClaim {
	day: number;
	contents: Contents;
	coins: number;
	tickets: TicketDto[];
	progress: Progress;
}

export interface ChatEvent {
	id: number;
	user: string;
	level: number;
	title: string | null;
	body: string;
	at: string;
}

export interface WinEvent {
	user: string;
	series: Series;
	prize: Prize;
	at: string;
}

export interface FeatEvent {
	user: string;
	/** The achievement earned */
	id: string;
	at: string;
}

export type FeedItem = ({ type: 'win' } & WinEvent) | ({ type: 'feat' } & FeatEvent);

export interface AnnouncementDto {
	id: number;
	body: string;
	at: string;
}

export interface Shouted {
	shout: ChatEvent;
	progress: Progress;
}

export interface ProfileDto {
	username: string;
	level: LevelDto;
	title: string | null;
	theme: ThemeId;
	joined: string;
	online: boolean;
	scratched: number;
	wins: number;
	best_win: number;
	best_streak: number;
	/** Achievements earned, newest first */
	achievements: string[];
	/** The viewer muted them */
	muted: boolean;
}

export interface PlayingCard {
	/** 2 to 14 (the ace) */
	rank: number;
	suit: 'spades' | 'hearts' | 'diamonds' | 'clubs';
}

export interface HighCardLayout {
	dealer: PlayingCard;
	hands: { card: PlayingCard; coins: number }[];
}

export type ReelSymbol = 'cherry' | 'lemon' | 'bell' | 'bar' | 'seven';

export interface ReelsLayout {
	/** Reel by reel, top to bottom */
	reels: ReelSymbol[][];
	/** Paylines that pay: rows 0–2, then the two diagonals */
	lines: number[];
}

export interface BingoLayout {
	/** Row by row; 0 is the free middle */
	card: number[][];
	calls: number[];
	lines: number[];
}

export type Spot = { kind: 'sand' } | { kind: 'chest'; coins: number } | { kind: 'map' };

export interface DigState {
	dug: { cell: number; spot: Spot }[];
	digs_left: number;
	/** After the last dig: the whole beach and the prize */
	done: Scratched | null;
}
