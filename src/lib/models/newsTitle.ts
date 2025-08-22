export type NewsTitle = {
  source: string;
  title: string;
  url: string;
  pics: string[];
  videos: string[];
};

export const SourceNames = {
  PengPai: "pengpai",
  ITHome: "ithome",
  // TouTiao: "toutiao",
  // WallStreetcn: "wallstreetcn",
};

export function sourceNameToDisplayName(sourceName: string): string {
  switch (sourceName) {
    case SourceNames.PengPai:
      return "澎湃新闻";
    // case SourceNames.TouTiao:
    //   return "今日头条";
    case SourceNames.ITHome:
      return "IT 之家";
    default:
      return "";
  }
}
