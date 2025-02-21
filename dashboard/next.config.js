/*
 * Copyright 2024 RisingWave Labs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

/**
 * @type {import('next').NextConfig}
 */
const nextConfig = {
  trailingSlash: true,

  rewrites: () => {
    return [
      {
        source: "/api/:path*",
        // To test with a RisingWave Meta node, use "http://127.0.0.1:5691/api/:path*"
        destination: "http://localhost:32333/:path*",
      },
    ]
  },
}

module.exports = nextConfig
