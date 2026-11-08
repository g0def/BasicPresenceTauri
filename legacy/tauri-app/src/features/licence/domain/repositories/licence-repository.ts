export interface LicenceRepository {
  /** The app licence as backend-rendered, sanitized HTML. */
  getHtml(): Promise<string>;
}
