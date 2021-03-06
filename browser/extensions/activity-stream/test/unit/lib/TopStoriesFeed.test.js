"use strict";
const injector = require("inject!lib/TopStoriesFeed.jsm");
const {FakePrefs} = require("test/unit/utils");
const {actionTypes: at} = require("common/Actions.jsm");
const {GlobalOverrider} = require("test/unit/utils");

describe("Top Stories Feed", () => {
  let TopStoriesFeed;
  let STORIES_UPDATE_TIME;
  let TOPICS_UPDATE_TIME;
  let SECTION_ID;
  let FEED_PREF;
  let instance;
  let clock;
  let globals;
  let sectionsManagerStub;
  let shortURLStub;

  const FAKE_OPTIONS = {
    "stories_endpoint": "https://somedomain.org/stories?key=$apiKey",
    "stories_referrer": "https://somedomain.org/referrer",
    "topics_endpoint": "https://somedomain.org/topics?key=$apiKey",
    "survey_link": "https://www.surveymonkey.com/r/newtabffx",
    "api_key_pref": "apiKeyPref",
    "provider_name": "test-provider",
    "provider_icon": "provider-icon",
    "provider_description": "provider_desc"
  };

  beforeEach(() => {
    FakePrefs.prototype.prefs.apiKeyPref = "test-api-key";

    globals = new GlobalOverrider();
    globals.set("Services", {locale: {getRequestedLocale: () => "en-CA"}});

    sectionsManagerStub = {
      onceInitialized: sinon.stub().callsFake(callback => callback()),
      enableSection: sinon.spy(),
      disableSection: sinon.spy(),
      updateSection: sinon.spy(),
      sections: new Map([["topstories", {options: FAKE_OPTIONS}]])
    };

    clock = sinon.useFakeTimers();

    shortURLStub = sinon.stub().callsFake(site => site.url);

    ({TopStoriesFeed, STORIES_UPDATE_TIME, TOPICS_UPDATE_TIME, SECTION_ID, FEED_PREF} = injector({
      "lib/ActivityStreamPrefs.jsm": {Prefs: FakePrefs},
      "lib/ShortURL.jsm": {shortURL: shortURLStub},
      "lib/SectionsManager.jsm": {SectionsManager: sectionsManagerStub}
    }));
    instance = new TopStoriesFeed();
    instance.store = {getState() { return {}; }, dispatch: sinon.spy()};
    instance.storiesLastUpdated = 0;
    instance.topicsLastUpdated = 0;
  });
  afterEach(() => {
    globals.restore();
    clock.restore();
  });
  describe("#init", () => {
    it("should create a TopStoriesFeed", () => {
      assert.instanceOf(instance, TopStoriesFeed);
    });
    it("should bind parseOptions to SectionsManager.onceInitialized", () => {
      instance.onAction({type: at.INIT});
      assert.calledOnce(sectionsManagerStub.onceInitialized);
    });
    it("should initialize endpoints based on options", () => {
      instance.onAction({type: at.INIT});
      assert.equal("https://somedomain.org/stories?key=test-api-key", instance.stories_endpoint);
      assert.equal("https://somedomain.org/referrer", instance.stories_referrer);
      assert.equal("https://somedomain.org/topics?key=test-api-key", instance.topics_endpoint);
    });
    it("should enable its section", () => {
      instance.onAction({type: at.INIT});
      assert.calledOnce(sectionsManagerStub.enableSection);
      assert.calledWith(sectionsManagerStub.enableSection, SECTION_ID);
    });
    it("should fetch stories on init", () => {
      instance.fetchStories = sinon.spy();
      instance.fetchTopics = sinon.spy();
      instance.onAction({type: at.INIT});
      assert.calledOnce(instance.fetchStories);
    });
    it("should fetch topics on init", () => {
      instance.fetchStories = sinon.spy();
      instance.fetchTopics = sinon.spy();
      instance.onAction({type: at.INIT});
      assert.calledOnce(instance.fetchTopics);
    });
    it("should not fetch if endpoint not configured", () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      sectionsManagerStub.sections.set("topstories", {options: {}});
      instance.init();
      assert.notCalled(fetchStub);
    });
    it("should report error for invalid configuration", () => {
      globals.sandbox.spy(global.Components.utils, "reportError");
      sectionsManagerStub.sections.set("topstories", {options: {api_key_pref: "invalid"}});
      instance.init();

      assert.called(Components.utils.reportError);
    });
    it("should report error for missing api key", () => {
      let fakeServices = {prefs: {getCharPref: sinon.spy()}, locale: {getRequestedLocale: sinon.spy()}};
      globals.set("Services", fakeServices);
      globals.sandbox.spy(global.Components.utils, "reportError");
      sectionsManagerStub.sections.set("topstories", {
        options: {
          "stories_endpoint": "https://somedomain.org/stories?key=$apiKey",
          "topics_endpoint": "https://somedomain.org/topics?key=$apiKey"
        }
      });
      instance.init();

      assert.called(Components.utils.reportError);
    });
    it("should initialize on FEED_INIT", () => {
      instance.init = sinon.spy();
      instance.onAction({type: at.FEED_INIT, data: FEED_PREF});
      assert.calledOnce(instance.init);
    });
  });
  describe("#uninit", () => {
    it("should disable its section", () => {
      instance.onAction({type: at.UNINIT});
      assert.calledOnce(sectionsManagerStub.disableSection);
      assert.calledWith(sectionsManagerStub.disableSection, SECTION_ID);
    });
  });
  describe("#fetch", () => {
    it("should fetch stories and send event", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      globals.set("NewTabUtils", {blockedLinks: {isBlocked: globals.sandbox.spy()}});

      const response = `{"recommendations": [{"id" : "1",
        "title": "title",
        "excerpt": "description",
        "image_src": "image-url",
        "url": "rec-url",
        "published_timestamp" : "123"
      }]}`;
      const stories = [{
        "guid": "1",
        "type": "now",
        "title": "title",
        "description": "description",
        "image": "image-url",
        "referrer": "referrer",
        "url": "rec-url",
        "eTLD": "",
        "hostname": "rec-url"
      }];

      instance.stories_endpoint = "stories-endpoint";
      instance.stories_referrer = "referrer";
      fetchStub.resolves({ok: true, status: 200, text: () => response});
      await instance.fetchStories();

      assert.calledOnce(fetchStub);
      assert.calledOnce(shortURLStub);
      assert.calledWithExactly(fetchStub, instance.stories_endpoint);
      assert.calledOnce(sectionsManagerStub.updateSection);
      assert.calledWith(sectionsManagerStub.updateSection, SECTION_ID, {rows: stories});
    });
    it("should call SectionsManager.updateSection", () => {
      instance.dispatchUpdateEvent(123, {});
      assert.calledOnce(sectionsManagerStub.updateSection);
    });
    it("should report error for unexpected stories response", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      globals.sandbox.spy(global.Components.utils, "reportError");

      instance.stories_endpoint = "stories-endpoint";
      fetchStub.resolves({ok: false, status: 400});
      await instance.fetchStories();

      assert.calledOnce(fetchStub);
      assert.calledWithExactly(fetchStub, instance.stories_endpoint);
      assert.notCalled(sectionsManagerStub.updateSection);
      assert.called(Components.utils.reportError);
    });
    it("should exclude blocked (dismissed) URLs", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      globals.set("NewTabUtils", {blockedLinks: {isBlocked: site => site.url === "blocked"}});

      const response = `{"recommendations": [{"url" : "blocked"}, {"url" : "not_blocked"}]}`;
      instance.stories_endpoint = "stories-endpoint";
      fetchStub.resolves({ok: true, status: 200, text: () => response});
      await instance.fetchStories();

      assert.calledOnce(sectionsManagerStub.updateSection);
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows.length, 1);
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows[0].url, "not_blocked");
    });
    it("should mark stories as new", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      globals.set("NewTabUtils", {blockedLinks: {isBlocked: globals.sandbox.spy()}});
      clock.restore();
      const response = JSON.stringify({
        "recommendations": [
          {"published_timestamp": Date.now() / 1000},
          {"published_timestamp": "0"},
          {"published_timestamp": (Date.now() - 2 * 24 * 60 * 60 * 1000) / 1000}
        ]
      });

      instance.stories_endpoint = "stories-endpoint";
      fetchStub.resolves({ok: true, status: 200, text: () => response});

      await instance.fetchStories();
      assert.calledOnce(sectionsManagerStub.updateSection);
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows.length, 3);
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows[0].type, "now");
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows[1].type, "trending");
      assert.equal(sectionsManagerStub.updateSection.firstCall.args[1].rows[2].type, "trending");
    });
    it("should fetch topics and send event", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);

      const response = `{"topics": [{"name" : "topic1", "url" : "url-topic1"}, {"name" : "topic2", "url" : "url-topic2"}]}`;
      const topics = [{
        "name": "topic1",
        "url": "url-topic1"
      }, {
        "name": "topic2",
        "url": "url-topic2"
      }];

      instance.topics_endpoint = "topics-endpoint";
      fetchStub.resolves({ok: true, status: 200, text: () => response});
      await instance.fetchTopics();

      assert.calledOnce(fetchStub);
      assert.calledWithExactly(fetchStub, instance.topics_endpoint);
      assert.calledOnce(sectionsManagerStub.updateSection);
      assert.calledWithMatch(sectionsManagerStub.updateSection, SECTION_ID, {topics});
    });
    it("should report error for unexpected topics response", async () => {
      let fetchStub = globals.sandbox.stub();
      globals.set("fetch", fetchStub);
      globals.sandbox.spy(global.Components.utils, "reportError");

      instance.topics_endpoint = "topics-endpoint";
      fetchStub.resolves({ok: false, status: 400});
      await instance.fetchTopics();

      assert.calledOnce(fetchStub);
      assert.calledWithExactly(fetchStub, instance.topics_endpoint);
      assert.notCalled(instance.store.dispatch);
      assert.called(Components.utils.reportError);
    });
  });
  describe("#update", () => {
    it("should fetch stories after update interval", () => {
      instance.init();
      instance.fetchStories = sinon.spy();
      instance.onAction({type: at.SYSTEM_TICK});
      assert.notCalled(instance.fetchStories);

      clock.tick(STORIES_UPDATE_TIME);
      instance.onAction({type: at.SYSTEM_TICK});
      assert.calledOnce(instance.fetchStories);
    });
    it("should fetch topics after update interval", () => {
      instance.init();
      instance.fetchTopics = sinon.spy();
      instance.onAction({type: at.SYSTEM_TICK});
      assert.notCalled(instance.fetchTopics);

      clock.tick(TOPICS_UPDATE_TIME);
      instance.onAction({type: at.SYSTEM_TICK});
      assert.calledOnce(instance.fetchTopics);
    });
  });
});
