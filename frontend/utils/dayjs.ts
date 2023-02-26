import dayjs from "dayjs/esm";

import weekOfYear from "dayjs/esm/plugin/weekOfYear";
import weekday from "dayjs/esm/plugin/weekday";
import dayOfYear from "dayjs/esm/plugin/dayOfYear";
import localeData from "dayjs/esm/plugin/localeData";
import relativeTime from "dayjs/esm/plugin/relativeTime";

import "dayjs/locale/pl";

dayjs.extend(weekOfYear);
dayjs.extend(weekday);
dayjs.extend(dayOfYear);
dayjs.extend(localeData);
dayjs.extend(relativeTime);

dayjs.locale("pl");

export default dayjs;
