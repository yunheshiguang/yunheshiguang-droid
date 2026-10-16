export type AppBuildInfo = Readonly<{
  versionCode: string;
  versionName: string;
}>;

export const appBuildInfo: AppBuildInfo = __APP_BUILD_INFO__;
