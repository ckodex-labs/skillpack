# gRPC API

SkillPack provides a production-ready gRPC API with 4 services.

## Services Overview

| Service          | Purpose                                        |
| ---------------- | ---------------------------------------------- |
| `SkillPackService` | Core assessment & grading                      |
| `IndexService`   | Skills Index management (like S&P 500)         |
| `RatingsService` | Moody's-style credit ratings                   |
| `QueryService`   | Search, CanIUse compatibility, cost estimation |

## SkillPackService

```protobuf
service SkillPackService {
  rpc Assess(AssessRequest) returns (AssessResponse);
  rpc Grade(GradeRequest) returns (GradeResponse);
  rpc Report(ReportRequest) returns (ReportResponse);
  rpc AssessStream(AssessRequest) returns (stream AssessEvent);
}
```

## IndexService

```protobuf
service IndexService {
  rpc CreateIndex(CreateIndexRequest) returns (CreateIndexResponse);
  rpc GetIndex(GetIndexRequest) returns (GetIndexResponse);
  rpc ListIndices(ListIndicesRequest) returns (ListIndicesResponse);
  rpc AddConstituent(AddConstituentRequest) returns (AddConstituentResponse);
  rpc CalculateValue(CalculateValueRequest) returns (CalculateValueResponse);
  rpc GetHistory(GetHistoryRequest) returns (GetHistoryResponse);
}
```

### Index Messages

```protobuf
message SkillsIndex {
  string id = 1;
  string name = 2;
  string description = 3;
  repeated IndexConstituent constituents = 5;
}

message IndexConstituent {
  string skill_ref = 1;
  double weight = 2;
  optional double quality_score = 4;
  optional string moodys_rating = 10;
  optional string carbon_rating = 8;
  optional double monthly_cost_usd = 9;
}

message IndexValue {
  double value = 2;
  string grade = 5;
}
```

## RatingsService

```protobuf
service RatingsService {
  rpc GetRating(GetRatingRequest) returns (GetRatingResponse);
  rpc GetRatingHistory(GetRatingHistoryRequest) returns (GetRatingHistoryResponse);
  rpc CompareRatings(CompareRatingsRequest) returns (CompareRatingsResponse);
}
```

### Ratings Scale

| Rating    | Category           | Investment Grade |
| --------- | ------------------ | ---------------- |
| Aaa       | Prime              | Yes              |
| Aa1-Aa3   | High Grade         | Yes              |
| A1-A3     | Upper Medium       | Yes              |
| Baa1-Baa3 | Lower Medium       | Yes              |
| Ba1-Ba3   | Speculative        | No               |
| B1-B3     | Highly Speculative | No               |
| Caa-C     | Substantial Risk   | No               |

## QueryService

```protobuf
service QueryService {
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc GetSkillProfile(GetSkillProfileRequest) returns (GetSkillProfileResponse);
  rpc GetCompatibility(GetCompatibilityRequest) returns (GetCompatibilityResponse);
  rpc GetCostEstimate(GetCostEstimateRequest) returns (GetCostEstimateResponse);
}
```

### CanIUse Compatibility

```protobuf
message SkillCompatibility {
  repeated CapabilitySupport capabilities = 2;
  uint32 compatibility_score = 3;
}

enum SupportLevel {
  SUPPORT_LEVEL_FULL = 1;
  SUPPORT_LEVEL_PARTIAL = 2;
  SUPPORT_LEVEL_NONE = 3;
}
```

### Cost Estimation

```protobuf
message CostEstimate {
  double monthly_cost_usd = 2;
  string cost_tier = 3;  // Free, Low, Medium, High, Enterprise
  CarbonFootprint carbon = 4;
}

message CarbonFootprint {
  double co2_grams_per_invocation = 1;
  string carbon_rating = 3;  // A, B, C, D, F
}
```

## Usage Examples

### grpcurl

```bash
# Assess skill
grpcurl -d '{"skill_path": "."}' \
  localhost:50051 skillpack.v1.SkillPackService/Assess

# Get index value
grpcurl -d '{"index_id": "skillpack-100"}' \
  localhost:50051 skillpack.v1.IndexService/CalculateValue

# Get skill rating
grpcurl -d '{"skill_ref": "my-skill"}' \
  localhost:50051 skillpack.v1.RatingsService/GetRating

# Search skills
grpcurl -d '{"min_grade": "A", "requires_capabilities": ["mcp-server"]}' \
  localhost:50051 skillpack.v1.QueryService/Search
```

### Rust Client

```rust
use skillpack_api::proto::{
    skill_pack_service_client::SkillPackServiceClient,
    index_service_client::IndexServiceClient,
    ratings_service_client::RatingsServiceClient,
    query_service_client::QueryServiceClient,
};

// Connect
let mut qa = SkillPackServiceClient::connect("http://localhost:50051").await?;
let mut index = IndexServiceClient::connect("http://localhost:50051").await?;
let mut ratings = RatingsServiceClient::connect("http://localhost:50051").await?;
let mut query = QueryServiceClient::connect("http://localhost:50051").await?;

// Assess
let response = qa.assess(AssessRequest { skill_path: ".".into(), .. }).await?;

// Get rating
let rating = ratings.get_rating(GetRatingRequest { skill_ref: "my-skill".into() }).await?;
println!("Rating: {} ({})", rating.rating.rating, rating.rating.outlook);
```

## Proto Location

```
proto/
└── skillpack/
    └── v1/
        └── skillpack.proto  # All 4 services + ~70 message types
```
