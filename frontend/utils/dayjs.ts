import dayjs from "dayjs";

import weekOfYear from "dayjs/plugin/weekOfYear";
import weekday from "dayjs/plugin/weekday";
import dayOfYear from "dayjs/plugin/dayOfYear";
import localeData from "dayjs/plugin/localeData";

import "dayjs/locale/pl";

dayjs.extend(weekOfYear);
dayjs.extend(weekday);
dayjs.extend(dayOfYear);
dayjs.extend(localeData);

dayjs.locale("pl");

export default dayjs;
