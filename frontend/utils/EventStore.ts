import { CalendarEvent } from "./CalendarEvent";
import dayjs from "./dayjs";

// TODO: implement overrides
export interface EventEntry {
    eventID: string;
    startTime: dayjs.Dayjs;
    endTime: dayjs.Dayjs;
}

export type ProcessedEvent = CalendarEvent & {
    raw: CalendarEvent;
    entry: EventEntry;
};

class EventIteratorContext {
    private store: EventStore;
    private startAt: number;
    private endAt: number;

    constructor(store: EventStore) {
        this.store = store;
        this.startAt = 0;
        this.endAt = -1;
    }

    // Sets the starting point. Can be a date or event index. Inclusive. Default 0.
    from(startPoint: dayjs.Dayjs | number) {
        if (typeof startPoint != "number") {
            this.startAt = this.store.findIndexAfter(startPoint) ?? this.store.entries.length;
        } else {
            this.startAt = startPoint;
        }
        return this;
    }

    // Sets the ending point. Can be a date or event index. Exclusive. Default -1 (end of the list)
    to(endPoint: dayjs.Dayjs | number) {
        if (typeof endPoint != "number") {
            const val = this.store.findIndexBefore(endPoint);
            if (val == null) {
                this.endAt = 0;
            } else {
                this.endAt = val + 1;
            }
        } else {
            this.endAt = endPoint;
        }
        return this;
    }

    // Sets the ending point by adding a number to the starting point.
    next(count: number) {
        this.endAt = this.startAt + count;
        return this;
    }

    *[Symbol.iterator]() {
        let startIndex = this.startAt;

        let endIndex: number;
        if (this.endAt < 0) {
            endIndex = this.store.entries.length;
        } else {
            endIndex = this.endAt;
        }

        if (startIndex > endIndex) {
            throw new RangeError("Invalid search bounds");
        }

        for (let i = startIndex; i < endIndex; i++) {
            const entry = this.store.entries[i];
            const original = this.store.data.get(entry.eventID)!;
            const incomplete: Partial<ProcessedEvent> = original.clone();
            incomplete.raw = original;
            incomplete.entry = entry;

            const event = incomplete as ProcessedEvent;
            incomplete.startTime = entry.startTime;
            incomplete.endTime = entry.endTime;
            // TODO: implement overrides
            yield event;
        }
    }
}

export class EventStore {
    // Chronological list of entries
    entries: EventEntry[];
    data: Map<string, CalendarEvent>;

    constructor() {
        // TODO: example data
        // TODO^2: fetch from API
        this.entries = [];
        this.data = new Map();
    }

    findIndexBefore(timestamp: dayjs.Dayjs): number | null {
        // TODO: test
        let start = 0;
        let end = this.entries.length - 1;
        while (start <= end) {
            // get the middle
            const mid = Math.floor((start + end) / 2);

            const el = this.entries[mid];
            const next = this.entries[mid + 1];
            if (!next) {
                // the last element may be appropriate
                return el.startTime.unix() < timestamp.unix() ? mid : null;
            }

            if (el.startTime.unix() < timestamp.unix() && next.startTime.unix() >= timestamp.unix()) {
                return mid;
            } else if (el.startTime.unix() < timestamp.unix()) {
                // both less
                start = mid + 1;
            } else {
                // both more (next is guaranteed to be after el)
                end = mid - 1;
            }
        }
        return null;
    }

    // This is really "not before" - it will find a value exactly at the timestamp if one exists
    findIndexAfter(timestamp: dayjs.Dayjs): number | null {
        if (this.entries.length > 0 && this.entries[0].startTime.unix() >= timestamp.unix()) {
            return 0;
        }

        const result = this.findIndexBefore(timestamp);
        if (result == null) return null;
        if (result + 1 == this.entries.length) {
            return null;
        }
        return result + 1;
    }

    iter() {
        return new EventIteratorContext(this);
    }

    // Bounds of date range that is loaded. Start is inclusive, end is exclusive.
    get loadedBounds() {
        return {
            start: this.entries[0].startTime,
            end: this.entries[this.entries.length - 1].endTime.add(1, "second"),
        }
    }

    // Load enough events so everything from newStart to the existing end is loaded.
    extendFrom(newStart: dayjs.Dayjs) {
        // TODO: request events in [newStart, loadedBounds.start)
    }

    // Load enough events so everything from the existing start to newEnd is loaded.
    extendTo(newEnd: dayjs.Dayjs) {
        // TODO: request events in [loadedBounds.end, newEnd)
    }
}

export interface EventStoreAPIData {
    entries: { eventID: string; startTime: number; endTime: number }[];
    data: Record<string, { name: string; description: string; startTime: number; endTime: number }>;
}

export function makeEventStore(data: EventStoreAPIData) {
    const store = reactive(new EventStore());
    store.entries = data.entries.map((entry) => ({
        eventID: entry.eventID,
        startTime: dayjs.unix(entry.startTime),
        endTime: dayjs.unix(entry.endTime),
    }));

    for (const [uuid, value] of Object.entries(data.data)) {
        store.data.set(
            uuid,
            new CalendarEvent(value.name, value.description, dayjs.unix(value.startTime), dayjs.unix(value.endTime))
        );
    }
    return store;
}
