/**
 * 文件名称：processIcon.ts
 *
 * 文件功能：
 * 按进程名选择 Devicon 品牌图标或 Font Awesome 回退图标。
 *
 * 主要职责：
 * - Redis / MySQL / Node / Nginx 等常见进程用 Devicon
 * - 未识别进程回退到 Font Awesome
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import {
  faCode,
  faDatabase,
  faGlobe,
  faMicrochip,
  faServer,
} from "@fortawesome/free-solid-svg-icons";
import apacheIcon from "devicon/icons/apache/apache-original.svg";
import apachekafkaIcon from "devicon/icons/apachekafka/apachekafka-original.svg";
import bunIcon from "devicon/icons/bun/bun-original.svg";
import chromeIcon from "devicon/icons/chrome/chrome-original.svg";
import denojsIcon from "devicon/icons/denojs/denojs-original.svg";
import dockerIcon from "devicon/icons/docker/docker-original.svg";
import dotnetcoreIcon from "devicon/icons/dotnetcore/dotnetcore-original.svg";
import elasticsearchIcon from "devicon/icons/elasticsearch/elasticsearch-original.svg";
import firefoxIcon from "devicon/icons/firefox/firefox-original.svg";
import goIcon from "devicon/icons/go/go-original.svg";
import grafanaIcon from "devicon/icons/grafana/grafana-original.svg";
import javaIcon from "devicon/icons/java/java-original.svg";
import kubernetesIcon from "devicon/icons/kubernetes/kubernetes-original.svg";
import mariadbIcon from "devicon/icons/mariadb/mariadb-original.svg";
import memcachedIcon from "devicon/icons/memcached/memcached-original.svg";
import mongodbIcon from "devicon/icons/mongodb/mongodb-original.svg";
import mysqlIcon from "devicon/icons/mysql/mysql-original.svg";
import nestjsIcon from "devicon/icons/nestjs/nestjs-original.svg";
import nextjsIcon from "devicon/icons/nextjs/nextjs-original.svg";
import nginxIcon from "devicon/icons/nginx/nginx-original.svg";
import nodejsIcon from "devicon/icons/nodejs/nodejs-original.svg";
import operaIcon from "devicon/icons/opera/opera-original.svg";
import phpIcon from "devicon/icons/php/php-original.svg";
import postgresqlIcon from "devicon/icons/postgresql/postgresql-original.svg";
import prometheusIcon from "devicon/icons/prometheus/prometheus-original.svg";
import pythonIcon from "devicon/icons/python/python-original.svg";
import rabbitmqIcon from "devicon/icons/rabbitmq/rabbitmq-original.svg";
import redisIcon from "devicon/icons/redis/redis-original.svg";
import rubyIcon from "devicon/icons/ruby/ruby-original.svg";
import rustIcon from "devicon/icons/rust/rust-original.svg";
import safariIcon from "devicon/icons/safari/safari-original.svg";
import sqliteIcon from "devicon/icons/sqlite/sqlite-original.svg";
import tomcatIcon from "devicon/icons/tomcat/tomcat-original.svg";
import traefikIcon from "devicon/icons/traefikproxy/traefikproxy-original.svg";
import vitejsIcon from "devicon/icons/vitejs/vitejs-original.svg";
import vscodeIcon from "devicon/icons/vscode/vscode-original.svg";
import webpackIcon from "devicon/icons/webpack/webpack-original.svg";

export type ProcessIconStyle =
  | {
      kind: "devicon";
      src: string;
      color: string;
      soft: string;
    }
  | {
      kind: "fa";
      icon: IconDefinition;
      color: string;
      soft: string;
    };

interface DeviconRule {
  test: RegExp;
  src: string;
  color: string;
  soft: string;
}

interface FaRule {
  test: RegExp;
  icon: IconDefinition;
  color: string;
  soft: string;
}

const DEVICON_RULES: DeviconRule[] = [
  { test: /redis/i, src: redisIcon, color: "#dc382d", soft: "var(--pk-stat-orange-soft)" },
  { test: /mariadb/i, src: mariadbIcon, color: "#c49a6c", soft: "var(--pk-stat-orange-soft)" },
  { test: /mysqld?/i, src: mysqlIcon, color: "#4479a1", soft: "var(--pk-stat-blue-soft)" },
  { test: /mongo/i, src: mongodbIcon, color: "#47a248", soft: "var(--pk-stat-green-soft)" },
  { test: /postgres|postgresql/i, src: postgresqlIcon, color: "#4169e1", soft: "var(--pk-stat-blue-soft)" },
  { test: /sqlite/i, src: sqliteIcon, color: "#003b57", soft: "var(--pk-stat-blue-soft)" },
  { test: /elastic/i, src: elasticsearchIcon, color: "#005571", soft: "var(--pk-stat-blue-soft)" },
  { test: /memcached/i, src: memcachedIcon, color: "#36b1bf", soft: "var(--pk-stat-blue-soft)" },
  { test: /rabbitmq/i, src: rabbitmqIcon, color: "#ff6600", soft: "var(--pk-stat-orange-soft)" },
  { test: /kafka/i, src: apachekafkaIcon, color: "#231f20", soft: "var(--pk-muted-bg)" },
  { test: /nginx/i, src: nginxIcon, color: "#009639", soft: "var(--pk-stat-green-soft)" },
  { test: /httpd|apache/i, src: apacheIcon, color: "#d22128", soft: "var(--pk-stat-orange-soft)" },
  { test: /tomcat/i, src: tomcatIcon, color: "#f8dc75", soft: "var(--pk-stat-orange-soft)" },
  { test: /traefik/i, src: traefikIcon, color: "#24a1c1", soft: "var(--pk-stat-blue-soft)" },
  { test: /docker/i, src: dockerIcon, color: "#2496ed", soft: "var(--pk-stat-blue-soft)" },
  { test: /kube/i, src: kubernetesIcon, color: "#326ce5", soft: "var(--pk-stat-blue-soft)" },
  { test: /deno/i, src: denojsIcon, color: "#000000", soft: "var(--pk-muted-bg)" },
  { test: /\bbun\b/i, src: bunIcon, color: "#fbf0df", soft: "var(--pk-stat-orange-soft)" },
  { test: /node/i, src: nodejsIcon, color: "#339933", soft: "var(--pk-stat-green-soft)" },
  { test: /python|uvicorn|gunicorn|pypy/i, src: pythonIcon, color: "#3776ab", soft: "var(--pk-stat-blue-soft)" },
  { test: /php/i, src: phpIcon, color: "#777bb4", soft: "var(--pk-stat-purple-soft)" },
  { test: /ruby|puma|rack/i, src: rubyIcon, color: "#cc342d", soft: "var(--pk-stat-orange-soft)" },
  { test: /java/i, src: javaIcon, color: "#007396", soft: "var(--pk-stat-blue-soft)" },
  { test: /dotnet|w3wp/i, src: dotnetcoreIcon, color: "#512bd4", soft: "var(--pk-stat-purple-soft)" },
  { test: /(^|[^a-z])go([^a-z]|$)/i, src: goIcon, color: "#00add8", soft: "var(--pk-stat-blue-soft)" },
  { test: /rustc|\bcargo\b/i, src: rustIcon, color: "#dea584", soft: "var(--pk-stat-orange-soft)" },
  { test: /vite/i, src: vitejsIcon, color: "#646cff", soft: "var(--pk-stat-purple-soft)" },
  { test: /webpack/i, src: webpackIcon, color: "#8dd6f9", soft: "var(--pk-stat-blue-soft)" },
  { test: /nestjs|\bnest\b/i, src: nestjsIcon, color: "#e0234e", soft: "var(--pk-stat-orange-soft)" },
  { test: /next/i, src: nextjsIcon, color: "#000000", soft: "var(--pk-muted-bg)" },
  { test: /^code$|code helper|code - oss|vscodium/i, src: vscodeIcon, color: "#007acc", soft: "var(--pk-stat-blue-soft)" },
  { test: /chrome/i, src: chromeIcon, color: "#4285f4", soft: "var(--pk-stat-blue-soft)" },
  { test: /firefox/i, src: firefoxIcon, color: "#ff7139", soft: "var(--pk-stat-orange-soft)" },
  { test: /opera/i, src: operaIcon, color: "#ff1b2d", soft: "var(--pk-stat-orange-soft)" },
  { test: /safari/i, src: safariIcon, color: "#006cff", soft: "var(--pk-stat-blue-soft)" },
  { test: /grafana/i, src: grafanaIcon, color: "#f46800", soft: "var(--pk-stat-orange-soft)" },
  { test: /prometheus/i, src: prometheusIcon, color: "#e6522c", soft: "var(--pk-stat-orange-soft)" },
];

const FA_RULES: FaRule[] = [
  {
    test: /caddy|iis/i,
    icon: faServer,
    color: "#22c55e",
    soft: "var(--pk-stat-green-soft)",
  },
  {
    test: /msedge|brave|browser/i,
    icon: faGlobe,
    color: "#f59e0b",
    soft: "var(--pk-stat-orange-soft)",
  },
  {
    test: /cursor|code|deno|python|ruby|php|java|dotnet/i,
    icon: faCode,
    color: "#8b5cf6",
    soft: "var(--pk-stat-purple-soft)",
  },
  {
    test: /sql|db|cache/i,
    icon: faDatabase,
    color: "#3d7eff",
    soft: "var(--pk-stat-blue-soft)",
  },
];

const FALLBACK: ProcessIconStyle = {
  kind: "fa",
  icon: faMicrochip,
  color: "#3d7eff",
  soft: "var(--pk-stat-blue-soft)",
};

function normalizeName(name: string): string {
  return name.trim().replace(/\.exe$/i, "");
}

/** 根据进程名返回图标样式。能识别的用 Devicon，其余用 Font Awesome。 */
export function processIconOf(name: string): ProcessIconStyle {
  const normalized = normalizeName(name);
  for (const rule of DEVICON_RULES) {
    if (rule.test.test(normalized)) {
      return {
        kind: "devicon",
        src: rule.src,
        color: rule.color,
        soft: rule.soft,
      };
    }
  }
  for (const rule of FA_RULES) {
    if (rule.test.test(normalized)) {
      return {
        kind: "fa",
        icon: rule.icon,
        color: rule.color,
        soft: rule.soft,
      };
    }
  }
  return FALLBACK;
}

/** 空值或占位符统一显示为破折号。 */
export function displayText(value: string | number | null | undefined): string {
  if (value === null || value === undefined) {
    return "—";
  }
  const text = String(value).trim();
  if (text === "" || text === "-") {
    return "—";
  }
  return text;
}
