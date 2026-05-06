"use strict";
/**
 * World Factory - Events Module
 *
 * Timeline and historical event components and types
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TimelineComponent = exports.timelineApi = exports.TimelineApiClient = void 0;
__exportStar(require("./TimelineTypes"), exports);
var TimelineApiClient_1 = require("./TimelineApiClient");
Object.defineProperty(exports, "TimelineApiClient", { enumerable: true, get: function () { return TimelineApiClient_1.TimelineApiClient; } });
Object.defineProperty(exports, "timelineApi", { enumerable: true, get: function () { return TimelineApiClient_1.timelineApi; } });
var TimelineComponent_1 = require("./TimelineComponent");
Object.defineProperty(exports, "TimelineComponent", { enumerable: true, get: function () { return TimelineComponent_1.TimelineComponent; } });
//# sourceMappingURL=index.js.map